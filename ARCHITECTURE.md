# OpenSim Next - Architecture Documentation

## Service Port Map (SINGLE SOURCE OF TRUTH)

| Service | Protocol | Port | File Location | Purpose | Status |
|---------|----------|------|---------------|---------|--------|
| **HTTP Login (XML-RPC)** | HTTP | 9000 | `src/network/handlers/login.rs` | Second Life viewer login | ✅ Active |
| **UDP LLUDP Protocol** | UDP | 9001 | `src/main.rs:526` | Viewer world communication | ✅ Active |
| **Web Interface** | HTTP | 8080 | `src/network/web_client.rs` | Browser client | ✅ Active |
| **Admin API** | HTTP | 9200 | `src/network/admin_api.rs` | Database management | ✅ Active |
| **Monitoring API** | HTTP | 9100 | `src/main.rs` | Prometheus metrics | ✅ Active |
| **Hypergrid Service** | HTTP | 8002 | `src/network/hypergrid.rs` | Inter-grid communication | ✅ Active |

## Login Flow Configuration

### XML-RPC Response (in `src/network/handlers/login.rs:1636`)
```xml
<member><name>sim_ip</name><value><string>127.0.0.1</string></value></member>
<member><name>sim_port</name><value><i4>9001</i4></value></member>
```

### Login Sequence
1. **HTTP Login**: Viewer → `localhost:9000` (XML-RPC login request)
2. **Response**: Server tells viewer to connect UDP to `localhost:9001`
3. **UDP Connection**: Viewer → `localhost:9001` (UseCircuitCode, RegionHandshake, etc.)
4. **CAPS Services**: Viewer → `localhost:9000/CAPS/*` (capabilities)

## Critical Files to Check Before Changes

| File | What to Verify |
|------|----------------|
| `src/main.rs:526` | UDP socket bind port (currently 9001) |
| `src/network/handlers/login.rs:1636` | XML-RPC `sim_port` value (currently 9001) |
| `src/network/handlers/login.rs:1635` | XML-RPC `sim_ip` value (currently 127.0.0.1) |

## Change Protocol (MANDATORY)

Before making ANY port changes:

1. **Document Current State**: Update this file with current ports
2. **Identify Problem**: What exactly is failing?
3. **Plan Change**: Document what you're changing and why
4. **Update This File**: Modify the tables above
5. **Test**: Verify change works
6. **Commit**: Include architecture changes in commit

## Known Working Configurations

### Configuration A (Current - 2025-07-13)
- HTTP Login: 9000
- UDP LLUDP: 9001 
- XML-RPC sim_port: 9001
- **Status**: Testing in progress

### Configuration B (Previous Working)
- HTTP Login: 9000
- UDP LLUDP: 9001
- XML-RPC sim_port: 9000
- **Status**: Worked briefly but caused confusion

## Debugging Login Issues

When viewer stalls at X%:

1. **30% Stall**: Usually port mismatch or UDP connection failure
   - Check: `netstat -an | grep 9001` (UDP should be listening)
   - Check: XML-RPC response sim_port matches UDP bind port
   
2. **RegionHandshake Stall**: Usually packet parsing or ACK issues
   - Check: Server logs for incoming UDP packets
   - Check: Packet parsing logic for message types

3. **General Network Issues**:
   - Check: `ps aux | grep opensim-next` (server running?)
   - Check: `curl http://localhost:9000/` (login endpoint responding?)
   - Check: Firewall/network configuration

## Current Issues to Track

- [ ] 30% login stall investigation
- [ ] Port configuration validation
- [ ] UDP packet handling verification

---

**Last Updated**: 2025-07-13
**Next Review**: Before any port/network changes