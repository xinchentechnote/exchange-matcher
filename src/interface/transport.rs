pub trait TransportAdapter {
    fn start(&self, order_tx: Sender<OrderRequest>);
}


pub struct TcpAdapter;

impl TransportAdapter for TcpAdapter {
    fn start(&self, order_tx: Sender<OrderRequest>) {
        let listener = TcpListener::bind("127.0.0.1:8080").unwrap();
        for stream in listener.incoming() {
            let order_tx = order_tx.clone();
            thread::spawn(move || {
                let mut stream = stream.unwrap();
                let mut buf = [0; 1024];
                loop {
                    let bytes_read = stream.read(&mut buf).unwrap();
                    if bytes_read == 0 {
                        break;
                    }
                    let order_request = parse_order_request(&buf[..bytes_read]);
                    order_tx.send(order_request).unwrap();
                }
            })
        }
    }
}