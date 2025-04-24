// src/p2p/node.rs
use crate::{debug, error, info, warn}; // Use logging macros
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use std::sync::Arc; // For thread-safe reference counting
use crate::p2p::signaling_types::SignalingMessage; // Our defined message types
use futures_util::{StreamExt, SinkExt}; // Traits for WebSocket streams
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use url::Url;
use tokio::sync::mpsc; // For sending messages internally if needed later
use webrtc::ice_transport::ice_candidate::RTCIceCandidateInit; // Need this later

// Placeholder struct for a P2P node's state
// Placeholder struct for a P2P node's state
pub struct P2PNode {
    node_id: String,
    webrtc_api: webrtc::api::API,
    // Channel for sending signaling messages outwards (optional for now)
    // signaling_tx: Option<mpsc::Sender<SignalingMessage>>,
}

impl P2PNode {
    // Function to create a new node instance
    pub async fn new() -> Result<Self, String> {
        info!("Creating new P2PNode instance...");

        // Create a MediaEngine (WebRTC requirement, even without audio/video)
        let mut m = MediaEngine::default();
        // Register default codecs (might customize later if needed)
        m.register_default_codecs()
            .map_err(|e| format!("Failed to register default codecs: {}", e))?;

        // Create the API object
        let api = APIBuilder::new()
            .with_media_engine(m)
            // Can add .with_setting_engine(setting_engine) if needed
            .build();

        info!("WebRTC API initialized.");

        Ok(Self {
            node_id: format!("node_{}", rand::random::<u32>()),
            webrtc_api: api, // Store the API object
        })
    }

    // Function to connect to the signaling server
    async fn connect_signaling(
        &self,
        server_url: &str,
    ) -> Result<WebSocketStream<MaybeTlsStream<TcpStream>>, String> {
        // To this block:
        info!("Connecting to signaling server: {}", server_url);
        // We still parse to validate, but pass the original string slice to connect_async
        let _url = Url::parse(server_url) // Use _url to indicate it's just for validation now
            .map_err(|e| format!("Failed to parse signaling server URL: {}", e))?;
        let (ws_stream, response) = connect_async(server_url).await // <--- Pass &str directly
            .map_err(|e| format!("Failed to connect to signaling server: {}", e))?;

        info!("Successfully connected to signaling server. Response status: {}", response.status());
        Ok(ws_stream)
    }

    // Function to create a single PeerConnection (will need signaling later)
    pub async fn create_peer_connection(&self) -> Result<Arc<RTCPeerConnection>, String> {
        info!("Attempting to create a new PeerConnection...");

        // Prepare the configuration (can add ICE servers later)
        let config = RTCConfiguration::default();

        // Create the PeerConnection using the API object
        let peer_connection = self.webrtc_api.new_peer_connection(config).await
            .map_err(|e| format!("Failed to create PeerConnection: {}", e))?;

        // Wrap in Arc for shared ownership
        let peer_connection = Arc::new(peer_connection);

        // --- Setup basic event handlers (more handlers needed later) ---
        let pc_clone = Arc::clone(&peer_connection);
        peer_connection.on_peer_connection_state_change(Box::new(move |s: RTCPeerConnectionState| {
            info!("Peer Connection State has changed: {}", s);
            if s == RTCPeerConnectionState::Failed {
                error!("Peer Connection Failed. Closing connection...");
                // In a real app, you might trigger cleanup or reconnection logic here
                let pc_clone_inner = Arc::clone(&pc_clone);
                tokio::spawn(async move {
                    if let Err(err) = pc_clone_inner.close().await {
                        error!("Failed to close PeerConnection: {}", err);
                    }
                });
            }
            Box::pin(async {}) // Required return type for the handler
        }));

        info!("PeerConnection created successfully.");
        Ok(peer_connection)
    }

    pub async fn run(&mut self, signaling_server_url: &str) -> Result<(), String> {
        info!("P2PNode {} starting run loop...", self.node_id);
    
        // 1. Connect to the signaling server
        let mut ws_stream = self.connect_signaling(signaling_server_url).await?;
        info!("Signaling connection established.");
    
        // --- Example: Send a simple Hello/Identify message upon connection (Optional) ---
        // let identify_msg = SignalingMessage::Identify(self.node_id.clone()); // Assuming Identify variant exists
        // let identify_json = serde_json::to_string(&identify_msg)
        //     .map_err(|e| format!("Failed to serialize identify message: {}", e))?;
        // if let Err(e) = ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(identify_json)).await {
        //     error!("Failed to send identify message: {}", e);
        // }
        // info!("Sent identify message.");
        // --- End Example ---
    
    
        // 2. Main loop to process incoming signaling messages
        loop {
            tokio::select! {
                // Read messages from WebSocket
                Some(msg_result) = ws_stream.next() => {
                    match msg_result {
                        // Replace the entire inner match msg { ... } block inside Ok(msg) => { ... }
                        Ok(msg) => {
                            if msg.is_text() {
                                match msg.to_text() { // Convert to &str if possible
                                    Ok(text_str) => {
                                        debug!("Received text message: {}", text_str);
                                        match serde_json::from_str::<SignalingMessage>(text_str) {
                                            Ok(signal_msg) => {
                                                info!("Parsed signaling message: {:?}", signal_msg);
                                                self.handle_signaling_message(signal_msg).await?;
                                            }
                                            Err(e) => {
                                                error!("Failed to parse JSON: {}. Raw: {}", e, text_str);
                                            }
                                        }
                                    }
                                    Err(e) => {
                                        // This might happen if the text message is fragmented and not complete UTF-8 yet
                                        error!("Failed to convert message to text (may be fragmented?): {:?}", e);
                                    }
                                }
                            } else if msg.is_binary() {
                                warn!("Received unexpected binary message: {:?}", msg.into_data());
                            } else if msg.is_ping() {
                                debug!("Received Ping frame");
                            } else if msg.is_pong() {
                                debug!("Received Pong frame");
                            } else if msg.is_close() {
                                info!("Received Close frame: {:?}", msg);
                                return Err("Signaling connection closed by peer.".to_string());
                            }
                            // Implicitly ignore raw frames or other types for now
                            }
                        Err(e) => {
                            error!("Error reading from WebSocket stream: {}", e);
                            // Attempt to reconnect or terminate based on error type
                            return Err(format!("Signaling connection error: {}", e));
                        }
                    }
                }
    
                // TODO: Add other branches to tokio::select! later
                // e.g., handle messages from application logic, timers, etc.
    
                // If ws_stream.next() returns None, the stream is closed
                else => {
                    info!("WebSocket stream closed.");
                    break; // Exit the loop
                }
            }
        }
        warn!("P2PNode run loop exited.");
        Ok(())
    }

        // Placeholder function to process parsed signaling messages
    async fn handle_signaling_message(&self, message: SignalingMessage) -> Result<(), String> {
        match message {
            SignalingMessage::Offer(sdp_data) => {
                info!("Received Offer SDP: {:?}", sdp_data.sdp.chars().take(50).collect::<String>());
                // TODO: Create PeerConnection, set remote description, create answer
            }
            SignalingMessage::Answer(sdp_data) => {
                info!("Received Answer SDP: {:?}", sdp_data.sdp.chars().take(50).collect::<String>());
                // TODO: Set remote description on existing PeerConnection
            }
            SignalingMessage::IceCandidate(candidate_data) => {
                info!("Received ICE Candidate: {:?}", candidate_data.candidate);
                // TODO: Add ICE candidate to existing PeerConnection
                let candidate_init = RTCIceCandidateInit::try_from(candidate_data)?;
                // Need access to the relevant PeerConnection object here...
                warn!("ICE Candidate handling not fully implemented yet.");
            }
            SignalingMessage::Error(err_msg) => {
                error!("Received error message via signaling: {}", err_msg);
                // TODO: Handle application-level errors reported by server/peer
            }
            // Add handlers for other message types if defined
        }
        Ok(())
    }
}