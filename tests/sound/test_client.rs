#[cfg(test)]
mod init_config {
    use super::*;

    #[test]
    fn test_default() {
        // Test with non-existent file and validate passed in defaults
        // From init_config:
        /*
            let default_params = json::object! {
                "api_url": "http://rtjam-nation.com/api/1/",
                "ws_url": "ws://rtjam-nation.com/primus",
                "no_loopback": false
            };
        */
        let expected_api_url = "http://rtjam-nation.com/api/1/";
        let expected_ws_url = "ws://rtjam-nation.com/primus";
        let expected_no_loopback = false;

        let result = init_config(Some("custom_settings.json"));
        assert!(result.is_ok());
        let (api_url, ws_url, mac_address, no_loopback) = result.unwrap();
        assert_eq!(api_url, expected_api_url);
        assert_eq!(ws_url, expected_ws_url);
        assert!(!mac_address.is_empty());
        assert_eq!(no_loopback, expected_no_loopback);
    }

    #[test]
    fn test_bad_file_name() {
        // Test with custom config file
        let result = init_config(Some("Illegal*File$Name"));
        assert!(result.is_err());
        assert!(result.err().unwrap().to_string().starts_with("Invalid filename 'Illegal*File$Name'"));
    }
}

mod init_api_connection {
    use super::*;
    
    #[test]
    fn test_init_api_connection() {
        let api_url = "http://test.com";
        let mac = "00:11:22:33:44:55";
        let git_hash = "abc123";

        let result = init_api_connection(api_url, mac, git_hash);
        assert!(result.is_ok());
    }
}

mod init_websocket_thread {
    use super::*;

    #[test]
    fn test_init_websocket_thread() {
        use mocktopus::mocktopus;

        let token = "test_token";
        let ws_url = "ws://test.com";

        // Create mock for websocket_thread
        moctopus! {
            fn websocket_thread(
                token: &str,
                ws_url: &str, 
                ws_tx: mpsc::Sender<Value>,
                ws_rx: mpsc::Receiver<WebsockMessage>
            ) -> Result<(), BoxError>;
        }

        // Set up mock expectations
        let mock = websocket_thread_mock();
        mock.expect()
            .with(eq(token), eq(ws_url), any(), any())
            .returns(Ok(()));

        // Call the function under test
        let result = init_websocket_thread(token, ws_url);
        assert!(result.is_ok());

        // Verify channels work
        let (to_ws_tx, _from_ws_rx, websocket_handle) = result.unwrap();
        assert!(to_ws_tx.send(WebsockMessage::Chat(serde_json::json!({"message": "test"}))).is_ok());
        assert!(websocket_handle.join().is_ok());

        // Verify mock was called
        mock.checkpoint();
    }
}

mod init_hardware_control {
    use super::*;
    
    #[test]
    fn test_init_hardware_control() {
        let result = init_hardware_control();
        assert!(result.is_ok());
        
        let (light_option, _handle) = result.unwrap();
        // Light option should be Some if hardware lights are available
        // None if not available
        match has_lights() {
            true => assert!(light_option.is_some()),
            false => assert!(light_option.is_none())
        }
    }
}