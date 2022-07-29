use mc_server_ping::ServerStatus;

fn main() {
    let mut server = ServerStatus::new("ssh.hypnos.ws", 12001, None, None);
    server.query().unwrap();
    let response = server.to_json().unwrap();
    println!("{}", serde_json::to_string_pretty(&response).unwrap());
}
