# OpenSim Next - API Endpoints Map

## HTTP API Endpoints

### Login Server (Port 9000)
| Method | Endpoint | Handler | Purpose | Request Format | Response Format |
|--------|----------|---------|---------|----------------|-----------------|
| POST | `/` | `src/network/handlers/login.rs:handle_login()` | Second Life viewer login | XML-RPC | XML-RPC |
| GET | `/health` | `src/network/handlers/login.rs` | Health check | - | JSON |

### Web Interface (Port 8080)
| Method | Endpoint | Handler | Purpose | Request Format | Response Format |
|--------|----------|---------|---------|----------------|-----------------|
| GET | `/` | `src/network/web_client.rs` | Main web interface | - | HTML |
| GET | `/static/*` | `src/network/web_client.rs` | Static assets | - | File |
| WS | `/ws` | `src/network/websocket.rs` | WebSocket connection | WebSocket | WebSocket |

### Admin API (Port 9200)
| Method | Endpoint | Handler | Purpose | Request Format | Response Format |
|--------|----------|---------|---------|----------------|-----------------|
| POST | `/admin/users` | `src/network/admin_api.rs` | Create user | JSON | JSON |
| GET | `/admin/users` | `src/network/admin_api.rs` | List users | - | JSON |
| GET | `/admin/users/account` | `src/network/admin_api.rs` | Show user account | Query params | JSON |
| PUT | `/admin/users/password` | `src/network/admin_api.rs` | Reset password | JSON | JSON |
| PUT | `/admin/users/email` | `src/network/admin_api.rs` | Reset email | JSON | JSON |
| PUT | `/admin/users/level` | `src/network/admin_api.rs` | Set user level | JSON | JSON |
| DELETE | `/admin/users/delete` | `src/network/admin_api.rs` | Delete user | JSON | JSON |
| GET | `/admin/database/stats` | `src/network/admin_api.rs` | Database statistics | - | JSON |
| GET | `/admin/health` | `src/network/admin_api.rs` | Admin API health | - | JSON |

### Monitoring API (Port 9100)
| Method | Endpoint | Handler | Purpose | Request Format | Response Format |
|--------|----------|---------|---------|----------------|-----------------|
| GET | `/metrics` | `src/main.rs` | Prometheus metrics | - | Prometheus format |
| GET | `/health` | `src/main.rs` | Health status | - | JSON |
| GET | `/info` | `src/main.rs` | Instance information | - | JSON |

### CAPS Services (Port 9000/CAPS/*)
| Method | Endpoint | Handler | Purpose | Request Format | Response Format |
|--------|----------|---------|---------|----------------|-----------------|
| GET | `/CAPS/{session_id}` | `src/caps/mod.rs` | Capability seed | - | LLSD |
| POST | `/CAPS/{session_id}/EventQueue` | `src/caps/event_queue.rs` | Event queue polling | LLSD | LLSD |
| POST | `/CAPS/{session_id}/UpdateAgentInformation` | `src/caps/handlers.rs` | Agent updates | LLSD | LLSD |
| POST | `/CAPS/{session_id}/UpdateAgentLanguage` | `src/caps/handlers.rs` | Language settings | LLSD | LLSD |

### Hypergrid Service (Port 8002)
| Method | Endpoint | Handler | Purpose | Request Format | Response Format |
|--------|----------|---------|---------|----------------|-----------------|
| GET | `/hypergrid` | `src/network/hypergrid.rs` | Hypergrid info | - | JSON |
| POST | `/hypergrid/link` | `src/network/hypergrid.rs` | Link regions | JSON | JSON |

## UDP Protocol Messages (Port 9001)

### Incoming Messages (Viewer → Server)
| Message ID | Message Name | Handler | Purpose | Expected Response |
|------------|--------------|---------|---------|-------------------|
| 3 | UseCircuitCode | `src/main.rs:584` | Establish session | UseCircuitCodeAck |
| 149 | RegionHandshakeReply | `src/main.rs:792` | Acknowledge region info | Continue login sequence |
| 251 | AgentUpdate | `src/main.rs:793` | Movement data | - |
| 1 | StartPingCheck | `src/main.rs:796` | Network test | CompletePingCheck |
| 2 | CompletePingCheck | `src/main.rs` | Ping response | - |
| 249 | CompleteAgentMovement | `src/main.rs` | Finish teleport | AgentMovementComplete |

### Outgoing Messages (Server → Viewer)
| Message ID | Message Name | Function | Purpose | When Sent |
|------------|--------------|----------|---------|-----------|
| 4 | UseCircuitCodeAck | `create_use_circuit_code_ack_packet()` | Confirm session | After UseCircuitCode |
| 1 | StartPingCheck | `create_start_ping_check_packet()` | Network test | Immediately after ack |
| 148 | RegionHandshake | `create_region_handshake_packet()` | Region info | Login sequence start |
| 158 | EnableSimulator | `create_enable_simulator_packet()` | Enable simulation | After RegionHandshake |
| 159 | TeleportFinish | `create_teleport_finish_packet()` | Complete teleport | Login sequence |
| 250 | AgentMovementComplete | `create_agent_movement_complete_packet()` | Movement complete | Login sequence |
| 249 | CompleteAgentMovement | `create_complete_agent_movement_packet()` | Agent ready | Login sequence |
| 21 | LayerData | `create_layer_data_packet()` | Terrain data | For world rendering |
| 12 | ObjectUpdate | `create_object_update_packet()` | Avatar appearance | For avatar display |
| 6 | CoarseLocationUpdate | `create_coarse_location_update_packet()` | Position updates | Periodic |

## Authentication Flow

### Entry Points
1. **XML-RPC Login** → `src/network/handlers/login.rs:handle_login()`
2. **Database Auth** → `src/database/user_accounts.rs:authenticate_user_opensim()`
3. **Circuit Code Validation** → `src/login_session.rs:validate_circuit_code()`

### Exit Points
1. **Successful Login** → XML-RPC response with session data
2. **Failed Auth** → XML-RPC error response
3. **UDP Session Established** → Login sequence packets sent
4. **Login Complete** → Avatar appears in world

## Data Flow Diagrams

### Login Sequence Flow
```
Viewer → HTTP POST :9000 → handle_login() → authenticate_user_opensim() 
    ↓
XML-RPC Response (circuit_code, sim_port=9001) → Viewer
    ↓
Viewer → UDP :9001 → UseCircuitCode → validate_circuit_code()
    ↓
UseCircuitCodeAck → StartPingCheck → RegionHandshake → ... → Login Complete
```

### Database Flow
```
HTTP Request → LoginServer → UserAccountDatabase → PostgreSQL Primary
                                ↓ (fallback)
                           MariaDB Secondary → Response
```

---

**Maintenance Notes:**
- Update this file when adding new endpoints
- Include handler file and function names
- Document request/response formats
- Track message IDs for UDP protocol