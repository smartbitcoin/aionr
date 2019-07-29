/*******************************************************************************
 * Copyright (c) 2018-2019 Aion foundation.
 *
 *     This file is part of the aion network project.
 *
 *     The aion network project is free software: you can redistribute it
 *     and/or modify it under the terms of the GNU General Public License
 *     as published by the Free Software Foundation, either version 3 of
 *     the License, or any later version.
 *
 *     The aion network project is distributed in the hope that it will
 *     be useful, but WITHOUT ANY WARRANTY; without even the implied
 *     warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.
 *     See the GNU General Public License for more details.
 *
 *     You should have received a copy of the GNU General Public License
 *     along with the aion network project source files.
 *     If not, see <https://www.gnu.org/licenses/>.
 *
 ******************************************************************************/

#![warn(unused_extern_crates)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate serde_derive;
extern crate futures;
extern crate bincode;
extern crate rand;
extern crate state;
extern crate tokio;
extern crate tokio_codec;
extern crate tokio_threadpool;
extern crate acore_bytes;
extern crate aion_types;
extern crate uuid;
extern crate aion_version as version;
extern crate bytes;
extern crate byteorder;

#[cfg(test)]
mod test;
mod config;
mod route;
mod msg;
mod node;
mod event;
mod codec;
pub mod states;
pub mod handler;

use std::collections::{hash_map::DefaultHasher, HashMap};
use std::hash::Hash;
use std::hash::Hasher;
use std::io;
use std::net::Shutdown;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc,Mutex,RwLock};
use std::thread;
use std::time::{Duration, Instant};
use rand::{thread_rng, Rng};
use state::Storage;
use futures::sync::mpsc;
use futures::{Future, Stream};
use tokio::net::{TcpListener, TcpStream};
use tokio::prelude::*;
use tokio::runtime::{Runtime,TaskExecutor};
use tokio::timer::Interval;
use tokio_codec::{Decoder,Framed};
use tokio_threadpool::{Builder, ThreadPool};
use codec::Codec;
use route::VERSION;
use route::MODULE;
use route::ACTION;
use handler::handshake;
use handler::active_nodes;
use handler::external::Handler;
use states::STATE::ALIVE;
use states::STATE::DISCONNECTED;
use states::STATE::ISSERVER;
use states::STATE::CONNECTED;

pub use node::*;
pub use config::Config;

lazy_static! {
    static ref WORKERS: Storage<RwLock<Arc<Runtime>>> = Storage::new();
    static ref LOCAL: Storage<Node> = Storage::new();
    static ref CONFIG: Storage<Config> = Storage::new();
    static ref SOCKETS: Storage<Mutex<HashMap<u64, TcpStream>>> = Storage::new();
    static ref NODES: RwLock<HashMap<u64, Node>> = { RwLock::new(HashMap::new()) };
    static ref ENABLED: Storage<AtomicBool> = Storage::new();
    static ref TP: Storage<ThreadPool> = Storage::new();
    static ref HANDLERS: Storage<Handler> = Storage::new();
}

const RECONNECT_BOOT_NOEDS_INTERVAL: u64 = 10;
const RECONNECT_NORMAL_NOEDS_INTERVAL: u64 = 1;
const NODE_ACTIVE_REQ_INTERVAL: u64 = 10;

pub fn register(handler: Handler) { HANDLERS.set(handler); }

fn connect_peer(peer_node: Node) {
    trace!(target: "net", "Try to connect to node {}", peer_node.get_ip_addr());
    let node_hash = calculate_hash(&peer_node.get_node_id());
    remove_peer(node_hash);
    create_client(peer_node, handle);
}

/// messages with module code other than p2p module
/// should flow into external handlers
fn handle(node: &mut Node, req: ChannelBuffer) {
    match VERSION::from(req.head.ver) {
        VERSION::V0 => {
            match MODULE::from(req.head.ctrl) {
                MODULE::P2P => {
                    match ACTION::from(req.head.action) {
                        ACTION::DISCONNECT => {
                            trace!(target: "net", "DISCONNECT received.");
                        }
                        ACTION::HANDSHAKEREQ => {
                            handshake::receive_req(node, req);
                        }
                        ACTION::HANDSHAKERES => {
                            handshake::receive_res(node, req);
                        }
                        ACTION::PING => {
                            // ignore
                        }
                        ACTION::PONG => {
                            // ignore
                        }
                        ACTION::ACTIVENODESREQ => {
                            active_nodes::receive_req(node);
                        }
                        ACTION::ACTIVENODESRES => {
                            active_nodes::receive_res(node, req);
                        }
                        _ => {
                            error!(target: "net", "Invalid action {} received.", req.head.action);
                        }
                    };
                }
                MODULE::EXTERNAL => {
                    trace!(target: "net", "P2P SYNC message received.");
                    let handler = HANDLERS.get();
                    handler.handle(node, req);
                }
            }
        }
        VERSION::V1 => {
            handshake::send(node);
        }
        _ => {
            error!(target: "net", "invalid version code");
        }
    };
}

pub fn enable(cfg: Config) {

    WORKERS.set(RwLock::new(Arc::new(
        Runtime::new().expect("Tokio Runtime"),
    )));

    SOCKETS.set(Mutex::new(
        HashMap::new()
    ));

    let local_node_str = cfg.local_node.clone();
    let mut local_node = Node::new_with_node_str(local_node_str);

    local_node.net_id = cfg.net_id;
    info!(target: "net", "        node: {}@{}", local_node.get_node_id(), local_node.get_ip_addr());

    LOCAL.set(local_node.clone());
    ENABLED.set(AtomicBool::new(true));

    TP.set(
        Builder::new()
            .pool_size((cfg.max_peers * 3) as usize)
            .build(),
    );

    CONFIG.set(cfg);

    thread::sleep(Duration::from_secs(5));
    let rt = &WORKERS.get().read().expect("get_executor").clone();
    let executor = rt.executor();
    let local_addr = get_local_node().get_ip_addr();
    create_server(&executor, &local_addr, handle);

    let local_node = get_local_node();
    let local_node_id_hash = calculate_hash(&local_node.get_node_id());
    let config = get_config();
    let boot_nodes = load_boot_nodes(config.boot_nodes.clone());
    let max_peers_num = config.max_peers as usize;
    let client_ip_black_list = config.ip_black_list.clone();
    let sync_from_boot_nodes_only = config.sync_from_boot_nodes_only;

    let connect_boot_nodes_task = Interval::new(
        Instant::now(),
        Duration::from_secs(RECONNECT_BOOT_NOEDS_INTERVAL),
    ).for_each(move |_| {
        for boot_node in boot_nodes.iter() {
            let node_hash = calculate_hash(&boot_node.get_node_id());
            if let Some(node) = get_node(node_hash) {
                if node.state_code == DISCONNECTED.value() {
                    trace!(target: "net", "boot node reconnected: {}@{}", boot_node.get_node_id(), boot_node.get_ip_addr());
                    connect_peer(boot_node.clone());
                }
            } else {
                trace!(target: "net", "boot node loaded: {}@{}", boot_node.get_node_id(), boot_node.get_ip_addr());
                connect_peer(boot_node.clone());
            }
        }

        Ok(())
    }).map_err(|e| error!("interval errored; err={:?}", e));
    executor.spawn(connect_boot_nodes_task);

    let connect_normal_nodes_task = Interval::new(
        Instant::now(),
        Duration::from_secs(RECONNECT_NORMAL_NOEDS_INTERVAL),
    )
    .for_each(move |_| {
        let active_nodes_count = get_nodes_count(ALIVE.value());
        if !sync_from_boot_nodes_only && active_nodes_count < max_peers_num {
            if let Some(peer_node) = get_an_inactive_node() {
                let peer_node_id_hash = calculate_hash(&peer_node.get_node_id());
                if peer_node_id_hash != local_node_id_hash {
                    let peer_ip = peer_node.ip_addr.get_ip();
                    if !client_ip_black_list.contains(&peer_ip) {
                        connect_peer(peer_node);
                    }
                }
            };
        }

        Ok(())
    })
    .map_err(|e| error!("interval errored; err={:?}", e));
    executor.spawn(connect_normal_nodes_task);

    let activenodes_req_task = Interval::new(
        Instant::now(),
        Duration::from_secs(NODE_ACTIVE_REQ_INTERVAL),
    )
    .for_each(move |_| {
        active_nodes::send();
        Ok(())
    })
    .map_err(|e| error!("interval errored; err={:?}", e));
    executor.spawn(activenodes_req_task);
}

pub fn create_server(
    executor: &TaskExecutor,
    local_addr: &String,
    handle: fn(node: &mut Node, req: ChannelBuffer),
)
{
    if let Ok(addr) = local_addr.parse() {
        let listener = TcpListener::bind(&addr).expect("Failed to bind");
        let server = listener
            .incoming()
            .map_err(|e| error!(target: "net", "Failed to accept socket; error = {:?}", e))
            .for_each(move |socket| {
                socket
                    .set_recv_buffer_size(1 << 24)
                    .expect("set_recv_buffer_size failed");

                socket
                    .set_keepalive(Some(Duration::from_secs(30)))
                    .expect("set_keepalive failed");

                process_inbounds(socket, handle);

                Ok(())
            });
        executor.spawn(server);
    } else {
        error!(target: "net", "Invalid ip address: {}", local_addr);
    }
}

pub fn create_client(peer_node: Node, handle: fn(node: &mut Node, req: ChannelBuffer)) {
    let node_ip_addr = peer_node.get_ip_addr();
    if let Ok(addr) = node_ip_addr.parse() {
        let thread_pool = get_thread_pool();
        let node_id = peer_node.get_node_id();
        let connect = TcpStream::connect(&addr)
            .map(move |socket| {
                socket
                    .set_recv_buffer_size(1 << 24)
                    .expect("set_recv_buffer_size failed");

                socket
                    .set_keepalive(Some(Duration::from_secs(30)))
                    .expect("set_keepalive failed");

                process_outbounds(socket, peer_node, handle);
            })
            .map_err(
                move |e| error!(target: "net", "    node: {}@{}, {}", node_ip_addr, node_id, e),
            );
        thread_pool.spawn(connect);
    }
}

pub fn get_thread_pool() -> &'static ThreadPool { TP.get() }

pub fn get_config() -> &'static Config { CONFIG.get() }

pub fn load_boot_nodes(boot_nodes_str: Vec<String>) -> Vec<Node> {
    let mut boot_nodes = Vec::new();
    for boot_node_str in boot_nodes_str {
        if boot_node_str.len() != 0 {
            let mut boot_node = Node::new_with_node_str(boot_node_str.to_string());
            boot_node.is_from_boot_list = true;
            boot_nodes.push(boot_node);
        }
    }
    boot_nodes
}

pub fn get_local_node() -> &'static Node { LOCAL.get() }

pub fn disable() {
    ENABLED.get().store(false, Ordering::SeqCst);
    reset();
}

pub fn reset() {
    if let Ok(mut sockets) = SOCKETS.get().lock() {
        for (_, socket) in sockets.iter_mut() {
            if let Err(e) = socket.shutdown() {
                error!(target: "net", "Invalid socket， {}", e);
            }
        }
    }
    if let Ok(mut nodes_map) = NODES.write() {
        nodes_map.clear();
    }
}

pub fn get_peer(node_hash: u64) -> Option<TcpStream> {
    if let Ok(mut socktes_map) = SOCKETS.get().lock() {
        return socktes_map.remove(&node_hash);
    }

    None
}

pub fn add_peer(node: Node, socket: &TcpStream) {
    if let Ok(socket) = socket.try_clone() {
        if let Ok(mut sockets) = SOCKETS.get().lock() {
            match sockets.get(&node.node_hash) {
                Some(_) => {
                    warn!(target: "net", "Known node, ...");
                }
                None => {
                    if let Ok(mut peer_nodes) = NODES.write() {
                        let max_peers_num = CONFIG.get().max_peers as usize;
                        if peer_nodes.len() < max_peers_num {
                            match peer_nodes.get(&node.node_hash) {
                                Some(_) => {
                                    warn!(target: "net", "Known node...");
                                }
                                None => {
                                    sockets.insert(node.node_hash, socket);
                                    peer_nodes.insert(node.node_hash, node);
                                    return;
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    if let Err(e) = socket.shutdown(Shutdown::Both) {
        error!(target: "net", "{}", e);
    }
}

pub fn remove_peer(node_hash: u64) -> Option<Node> {
    if let Ok(mut sockets) = SOCKETS.get().lock() {
        if let Some(socket) = sockets.remove(&node_hash) {
            if let Err(e) = socket.shutdown(Shutdown::Both) {
                trace!(target: "net", "remove_peer， invalid socket， {}", e);
            }
        }
    }
    if let Ok(mut peer_nodes) = NODES.write() {
        // if let Some(node) = peer_nodes.remove(&node_hash) {
        //     info!(target: "p2p", "Node {}@{} removed.", node.get_node_id(), node.get_ip_addr());
        //     return Some(node);
        // }
        // info!(target: "net", "remove_peer， peer_node hash: {}", node_hash);
        return peer_nodes.remove(&node_hash);
    }

    None
}

pub fn add_node(node: Node) {
    let max_peers_num = CONFIG.get().max_peers as usize;
    if let Ok(mut nodes_map) = NODES.write() {
        if nodes_map.len() < max_peers_num {
            match nodes_map.get(&node.node_hash) {
                Some(_) => {
                    warn!(target: "net", "Known node...");
                }
                None => {
                    nodes_map.insert(node.node_hash, node);
                    return;
                }
            }
        }
    }
}

pub fn is_connected(node_id_hash: u64) -> bool {
    let all_nodes = get_all_nodes();
    for node in all_nodes.iter() {
        if node_id_hash == calculate_hash(&node.get_node_id()) {
            return true;
        }
    }
    false
}

pub fn get_nodes_count(state_code: u32) -> usize {
    let mut nodes_count = 0;
    if let Ok(nodes_map) = NODES.read() {
        for val in nodes_map.values() {
            if val.state_code & state_code == state_code {
                nodes_count += 1;
            }
        }
    }
    nodes_count
}

pub fn get_nodes_count_with_mode(mode: Mode) -> usize {
    let mut nodes_count = 0;
    if let Ok(nodes_map) = NODES.read() {
        for val in nodes_map.values() {
            if val.state_code & ALIVE.value() == ALIVE.value() && val.mode == mode {
                nodes_count += 1;
            }
        }
    }
    nodes_count
}

pub fn get_nodes_count_all_modes() -> (usize, usize, usize, usize, usize) {
    let mut normal_nodes_count = 0;
    let mut backward_nodes_count = 0;
    let mut forward_nodes_count = 0;
    let mut lightning_nodes_count = 0;
    let mut thunder_nodes_count = 0;
    if let Ok(nodes_map) = NODES.read() {
        for val in nodes_map.values() {
            if val.state_code & ALIVE.value() == ALIVE.value() {
                match val.mode {
                    Mode::NORMAL => normal_nodes_count += 1,
                    Mode::BACKWARD => backward_nodes_count += 1,
                    Mode::FORWARD => forward_nodes_count += 1,
                    Mode::LIGHTNING => lightning_nodes_count += 1,
                    Mode::THUNDER => thunder_nodes_count += 1,
                }
            }
        }
    }
    (
        normal_nodes_count,
        backward_nodes_count,
        forward_nodes_count,
        lightning_nodes_count,
        thunder_nodes_count,
    )
}

pub fn get_all_nodes_count() -> u16 {
    let mut count = 0;
    if let Ok(nodes_map) = NODES.read() {
        for _ in nodes_map.values() {
            count += 1;
        }
    }
    count
}

pub fn get_all_nodes() -> Vec<Node> {
    let mut nodes = Vec::new();
    if let Ok(nodes_map) = NODES.read() {
        for val in nodes_map.values() {
            let node = val.clone();
            nodes.push(node);
        }
    }
    nodes
}

pub fn get_nodes(state_code_mask: u32) -> Vec<Node> {
    let mut nodes = Vec::new();
    if let Ok(nodes_map) = NODES.read() {
        for val in nodes_map.values() {
            let node = val.clone();
            if node.state_code & state_code_mask == state_code_mask {
                nodes.push(node);
            }
        }
    }
    nodes
}

pub fn get_an_inactive_node() -> Option<Node> {
    let nodes = get_nodes(DISCONNECTED.value());
    let mut normal_nodes = Vec::new();
    for node in nodes.iter() {
        if node.is_from_boot_list {
            continue;
        } else {
            normal_nodes.push(node);
        }
    }
    let normal_nodes_count = normal_nodes.len();
    if normal_nodes_count == 0 {
        return None;
    }
    let mut rng = thread_rng();
    let random_index: usize = rng.gen_range(0, normal_nodes_count);
    let node = &normal_nodes[random_index];

    remove_peer(node.node_hash)
}

pub fn get_an_active_node() -> Option<Node> {
    let nodes = get_nodes(ALIVE.value());
    let node_count = nodes.len();
    if node_count > 0 {
        let mut rng = thread_rng();
        let random_index: usize = rng.gen_range(0, node_count);
        return get_node(nodes[random_index].node_hash);
    } else {
        None
    }
}

pub fn get_node(node_hash: u64) -> Option<Node> {
    if let Ok(nodes_map) = NODES.read() {
        if let Some(node) = nodes_map.get(&node_hash) {
            return Some(node.clone());
        }
    }
    None
}

pub fn update_node_with_mode(node_hash: u64, node: &Node) {
    if let Ok(mut nodes_map) = NODES.write() {
        if let Some(n) = nodes_map.get_mut(&node_hash) {
            n.update(node);
        }
    }
}

pub fn update_node(node_hash: u64, node: &mut Node) {
    if let Ok(mut nodes_map) = NODES.write() {
        if let Some(n) = nodes_map.get_mut(&node_hash) {
            node.mode = n.mode.clone();
            n.update(node);
        }
    }
}

pub fn process_inbounds(socket: TcpStream, handle: fn(node: &mut Node, req: ChannelBuffer)) {
    if let Ok(peer_addr) = socket.peer_addr() {
        let mut peer_node = Node::new_with_addr(peer_addr);
        let peer_ip = peer_node.ip_addr.get_ip();
        let local_ip = get_local_node().ip_addr.get_ip();
        let config = get_config();
        if get_nodes_count(ALIVE.value()) < config.max_peers as usize
            && !config.ip_black_list.contains(&peer_ip)
        {
            let mut value = peer_node.ip_addr.get_addr();
            value.push_str(&local_ip);
            peer_node.node_hash = calculate_hash(&value);
            peer_node.state_code = CONNECTED.value();
            trace!(target: "net", "New incoming connection: {}", peer_addr);

            let (tx, rx) = mpsc::channel(409600);
            let thread_pool = get_thread_pool();

            peer_node.tx = Some(tx);
            peer_node.state_code = CONNECTED.value();
            peer_node.ip_addr.is_server = false;

            trace!(target: "net", "A new peer added: {}", peer_node);

            let mut node_hash = peer_node.node_hash;
            add_peer(peer_node, &socket);
            // process request from the incoming stream
            let (sink, stream) = split_frame(socket);
            let read = stream.for_each(move |msg| {
                if let Some(mut peer_node) = get_node(node_hash) {
                    handle(&mut peer_node, msg.clone());
                    node_hash = calculate_hash(&peer_node.get_node_id());
                }

                Ok(())
            });

            thread_pool.spawn(read.then(|_| Ok(())));

            // send everything in rx to sink
            let write =
                sink.send_all(rx.map_err(|()| {
                    io::Error::new(io::ErrorKind::Other, "rx shouldn't have an error")
                }));
            thread_pool.spawn(write.then(move |_| {
                trace!(target: "net", "Connection with {:?} closed.", peer_ip);
                Ok(())
            }));
        }
    } else {
        error!(target: "net", "Invalid socket: {:?}", socket);
    }
}

fn process_outbounds(
    socket: TcpStream,
    peer_node: Node,
    handle: fn(node: &mut Node, req: ChannelBuffer),
)
{
    let mut peer_node = peer_node.clone();
    peer_node.node_hash = calculate_hash(&peer_node.get_node_id());
    let node_hash = peer_node.node_hash;

    if let Some(node) = get_node(node_hash) {
        if node.state_code == DISCONNECTED.value() {
            trace!(target: "net", "update known peer node {}@{}...", node.get_node_id(), node.get_ip_addr());
            remove_peer(node_hash);
        } else {
            return;
        }
    }

    let (tx, rx) = mpsc::channel(409600);
    peer_node.tx = Some(tx);
    peer_node.state_code = CONNECTED.value() | ISSERVER.value();
    peer_node.ip_addr.is_server = true;
    let peer_ip = peer_node.get_ip_addr().clone();
    trace!(target: "net", "A new peer added: {}@{}", peer_node.get_node_id(), peer_node.get_ip_addr());

    add_peer(peer_node.clone(), &socket);

    // process request from the outcoming stream
    let (sink, stream) = split_frame(socket);

    // OnConnect
    let mut req = ChannelBuffer::new();
    req.head.ver = VERSION::V1.value();
    handle(&mut peer_node, req);

    let read = stream.for_each(move |msg| {
        if let Some(mut peer_node) = get_node(node_hash) {
            handle(&mut peer_node, msg);
        }

        Ok(())
    });
    let thread_pool = get_thread_pool();
    thread_pool.spawn(read.then(|_| Ok(())));

    // send everything in rx to sink
    let write = sink.send_all(
        rx.map_err(|()| io::Error::new(io::ErrorKind::Other, "rx shouldn't have an error")),
    );
    thread_pool.spawn(write.then(move |_| {
        trace!(target: "net", "Connection with {:?} closed.", peer_ip);
        Ok(())
    }));
}

pub fn send(node_hash: u64, msg: ChannelBuffer) {
    match NODES.read() {
        Ok(nodes) => {
            match nodes.get(&node_hash) {
                Some(ref node) => {
                    let tx = node.tx.clone();
                    // tx should be contructed at begin lifecycle of any node in NODES
                    if tx.is_some() {
                        match tx.unwrap().try_send(msg) {
                            Ok(_) => {},
                            Err(err) => {
                                // TODO: dispatch node not found event for upper modules
                                remove_peer(node_hash);
                                trace!(target: "p2p", "fail sending msg, {}", err);
                            }
                        }
                    }
                }, 
                None => {
                    // TODO: dispatch node not found event for upper modules
                    remove_peer(node_hash);
                    trace!(target: "p2p", "peer not found, {}", node_hash);
                }
            }
        },
        Err(_err) => {
            // TODO: dispatch node not found event for upper modules
        }
    }
}

pub fn calculate_hash<T: Hash>(t: &T) -> u64 {
    let mut s = DefaultHasher::new();
    t.hash(&mut s);
    s.finish()
}

pub fn split_frame(
    socket: TcpStream,
) -> (
    stream::SplitSink<Framed<TcpStream, Codec>>,
    stream::SplitStream<Framed<TcpStream, Codec>>,
) {
    Codec.framed(socket).split()
}
