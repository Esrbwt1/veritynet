// src/p2p/node.rs
use crate::{debug, error, info}; // Use logging macros
use webrtc::api::APIBuilder;
use webrtc::api::media_engine::MediaEngine;
use webrtc::api::setting_engine::SettingEngine;
use webrtc::peer_connection::configuration::RTCConfiguration;
use webrtc::peer_connection::peer_connection_state::RTCPeerConnectionState;
use webrtc::peer_connection::RTCPeerConnection;
use std::sync::Arc; // For thread-safe reference counting

// Placeholder struct for a P2P node's state
// Placeholder struct for a P2P node's state
pub struct P2PNode {
    node_id: String,
    webrtc_api: webrtc::api::API, // Add this field
    // We'll add peer connections dynamically later
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

    // Placeholder run function (remains the same for now)
    pub async fn run(&mut self) -> Result<(), String> {
        info!("P2PNode {} starting run loop...", self.node_id);
        loop {
            debug!("P2PNode {} run loop iteration...", self.node_id);
            tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        }
        // Ok(())
    }
}