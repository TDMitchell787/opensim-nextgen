use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::atomic::{AtomicU32, Ordering};
use parking_lot::RwLock;
use tracing::{debug, warn};

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    pub address: SocketAddr,
    pub last_sequence: u32,
    pub agent_id: Option<uuid::Uuid>,
    pub ping_sequence: u8,
}

#[derive(Debug)]
pub struct ReliabilityManager {
    connections: Arc<RwLock<HashMap<u32, ConnectionInfo>>>, // circuit_code -> connection
    global_sequence: AtomicU32,
}

impl ReliabilityManager {
    pub fn new() -> Self {
        Self {
            connections: Arc::new(RwLock::new(HashMap::new())),
            global_sequence: AtomicU32::new(1),
        }
    }
    
    pub fn add_connection(&self, circuit_code: u32, address: SocketAddr) {
        let connection = ConnectionInfo {
            address,
            last_sequence: 0,
            agent_id: None,
            ping_sequence: 0,
        };

        let mut connections = self.connections.write();
        connections.insert(circuit_code, connection);

        debug!("Added UDP connection for circuit {} at {}", circuit_code, address);
    }

    pub fn set_agent_id(&self, circuit_code: u32, agent_id: uuid::Uuid) {
        let mut connections = self.connections.write();
        if let Some(connection) = connections.get_mut(&circuit_code) {
            connection.agent_id = Some(agent_id);
            debug!("Set agent_id {} for circuit {}", agent_id, circuit_code);
        }
    }
    
    pub fn remove_connection(&self, circuit_code: u32) {
        let mut connections = self.connections.write();
        if let Some(connection) = connections.remove(&circuit_code) {
            debug!("Removed UDP connection for circuit {} ({})", circuit_code, connection.address);
        }
    }
    
    pub fn get_connection(&self, circuit_code: u32) -> Option<ConnectionInfo> {
        self.connections.read().get(&circuit_code).cloned()
    }
    
    pub fn update_sequence(&self, circuit_code: u32, sequence: u32) {
        let mut connections = self.connections.write();
        if let Some(connection) = connections.get_mut(&circuit_code) {
            connection.last_sequence = sequence;
            debug!("Updated sequence for circuit {} to {}", circuit_code, sequence);
        }
    }
    
    pub fn next_sequence(&self) -> u32 {
        self.global_sequence.fetch_add(1, Ordering::SeqCst)
    }

    pub fn reserve_sequences(&self, count: u32) -> u32 {
        self.global_sequence.fetch_add(count, Ordering::SeqCst)
    }
    
    pub fn get_active_connections(&self) -> Vec<(u32, ConnectionInfo)> {
        self.connections
            .read()
            .iter()
            .map(|(k, v)| (*k, v.clone()))
            .collect()
    }
    
    pub fn get_connection_count(&self) -> usize {
        self.connections.read().len()
    }

    pub fn get_connection_by_addr(&self, addr: SocketAddr) -> Option<ConnectionInfo> {
        let connections = self.connections.read();
        connections.values().find(|c| c.address == addr).cloned()
    }

    pub fn get_circuit_code_by_addr(&self, addr: SocketAddr) -> Option<u32> {
        let connections = self.connections.read();
        connections.iter()
            .find(|(_, c)| c.address == addr)
            .map(|(code, _)| *code)
    }

    pub fn next_ping_sequence(&self, circuit_code: u32) -> u8 {
        let mut connections = self.connections.write();
        if let Some(connection) = connections.get_mut(&circuit_code) {
            let seq = connection.ping_sequence;
            connection.ping_sequence = connection.ping_sequence.wrapping_add(1);
            seq
        } else {
            0
        }
    }
}

impl Default for ReliabilityManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::net::{IpAddr, Ipv4Addr};

    #[test]
    fn test_connection_management() {
        let manager = ReliabilityManager::new();
        let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 12345);
        let circuit_code = 12345u32;
        
        assert_eq!(manager.get_connection_count(), 0);
        
        manager.add_connection(circuit_code, address);
        assert_eq!(manager.get_connection_count(), 1);
        
        let connection = manager.get_connection(circuit_code).unwrap();
        assert_eq!(connection.address, address);
        assert_eq!(connection.last_sequence, 0);
        
        manager.update_sequence(circuit_code, 100);
        let updated_connection = manager.get_connection(circuit_code).unwrap();
        assert_eq!(updated_connection.last_sequence, 100);
        
        manager.remove_connection(circuit_code);
        assert_eq!(manager.get_connection_count(), 0);
        assert!(manager.get_connection(circuit_code).is_none());
    }
    
    #[test]
    fn test_sequence_generation() {
        let manager = ReliabilityManager::new();
        
        let seq1 = manager.next_sequence();
        let seq2 = manager.next_sequence();
        let seq3 = manager.next_sequence();
        
        assert_eq!(seq1, 1);
        assert_eq!(seq2, 2);
        assert_eq!(seq3, 3);
    }
}