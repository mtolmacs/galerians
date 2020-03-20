use std::net::{ToSocketAddrs};
use std::process::Command;
use std::{thread, time};
use clap::{Arg, ArgGroup, App};
use mysql::*;
use mysql::prelude::*;


struct Parameters {
  pub domain: String,
  pub connstr: String,
  pub frequency: u64,
  pub port: u32,
  pub ignore_ips: Vec<String>
}

impl Parameters {
  /*
   * Creates a new Parameter instance by parsing the command line
   */
  pub fn new() -> Parameters {
    let args = App::new("Dynamic Galera Cluster Address Updater")
      .version("0.1.0")
      .author("Mark Tolmacs <mark@lazycat.hu>")
      .about("Updates the wsrep_cluster_address global parameter in a galera \
            node to match all the IPs a given \ndomain resolves to. Useful to \
            automatically add all possible galera nodes to each node.\n\
            \n\
            This program does not exit.")
      .arg(Arg::with_name("connection")
          .short("c")
          .long("connection")
          .takes_value(true)
          .help("A valid MySQL connection string"))
      .arg(Arg::with_name("file")
          .short("f")
          .long("file")
          .takes_value(true)
          .help("A path to a file containing a valid MySQL connection string"))
      .group(ArgGroup::with_name("mysql")
          .args(&["connection", "file"])
          .multiple(false)
          .required(true))
      .arg(Arg::with_name("domain")
          .short("d")
          .long("domain")
          .required(true)
          .takes_value(true)
          .help("The domain resolve continuously and monitor for IPs"))
      .arg(Arg::with_name("frequency")
          .short("q")
          .long("frequency")
          .takes_value(true)
          .help("Seconds to wait before querying the domain name"))
      .arg(Arg::with_name("port")
          .short("p")
          .long("port")
          .takes_value(true)
          .help("The Galera port on the nodes (defaults to 4567"))
      .get_matches();

    Parameters {
      domain: args.value_of("domain")
                  .unwrap()
                  .to_owned(),
      connstr: args.value_of("connection")
                   .unwrap()
                   .to_owned(),
      frequency:  args.value_of("frequency")
                      .unwrap_or("5")
                      .parse::<u64>()
                      .expect("Frequency has to be a positive number")
                      .to_owned(),
      port: args.value_of("port")
                .unwrap_or("4567")
                .parse::<u32>()
                .expect("Port has to be a valid port number")
                .to_owned(),
      ignore_ips: Self::get_local_ips(),
    }
  }

  /*
   * Get the IP(s) of the local machine.
   *
   * Original from https://github.com/ivanceras/machine-ip (MIT)
   */
  fn get_local_ips() -> Vec<String> {
    let output = match Command::new("hostname").args(&["-I"]).output() {
      Ok(ok) => ok,
      Err(_) => {
        return vec![];
      }
    };
  
    let stdout = match String::from_utf8(output.stdout) {
      Ok(ok) => ok,
      Err(_) => {
        return vec![];
      }
    };
  
    let ips: Vec<&str> = stdout.trim().split(" ").collect::<Vec<&str>>();
  
    return ips.into_iter().map(|ip| String::from(ip)).collect();
  }
}

/*
 * Initializes a connection to mysql and returns this connection.
 */
fn get_mysql_conn(mysql_str: &str) -> PooledConn {
  let mut pool_maybe = Pool::new(mysql_str);
  while pool_maybe.is_err() {
    thread::sleep(time::Duration::from_secs(1));
    println!("Waiting for MySQL server to become available...");  
    pool_maybe = Pool::new(&mysql_str[..]);
  }

  let pool = pool_maybe.unwrap();
  let mut conn = pool.get_conn();
  while conn.is_err() {
    thread::sleep(time::Duration::from_secs(1));
    println!("Waiting for MySQL connection...");
    conn = pool.get_conn();
  }
  
  println!("MySQL connection successfully established!");

  return conn.unwrap();
}

fn main() {
  let args = Parameters::new();
  let update_query = "SET @@global.wsrep_cluster_address = ?";
  let mut conn = get_mysql_conn(&args.connstr);
  let mut cluster_address = String::from("gcomm://");

  loop {
    thread::sleep(time::Duration::from_secs(args.frequency));
    match format!("{}:{}", args.domain, args.port).to_socket_addrs() {
      Ok(_iter) => {
        let addrs: Vec<String> = _iter.map(|item| item.ip().to_string())
            .filter(|item| match args.ignore_ips.iter().find(| &x | x == item) {
              Some(_) => false,
              None => true,
            })
            .map(|item| format!("{}:{}", item, args.port))
            .collect();
        let new_address = format!("gcomm://{}", addrs.join(","));
        if new_address != cluster_address {
          match conn.exec_drop(update_query, (&new_address[..],)) {
            Err(_error) => 
              println!("Error executing cluster address update: {}", _error),
            Ok(_) => 
              println!("dynamic_galera_cluster: {} -> {}", 
                       cluster_address, 
                       new_address),
          };
          
          cluster_address = new_address;
        }
      },
      Err(_e) => 
        println!("dynamic_galera_cluster: Unable to resolve domain: {:?}", _e),
    }
  }
}