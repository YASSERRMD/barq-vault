use crate::error::{BarqError, BarqResult};
use std::io;

#[test]
fn test_error_display() {
    let err = BarqError::Storage("test failure".to_string());
    assert_eq!(format!("{}", err), "Storage error: test failure");
    
    let err = BarqError::NotFound(uuid::Uuid::nil());
    assert_eq!(format!("{}", err), "Record not found: 00000000-0000-0000-0000-000000000000");
}

#[test]
fn test_error_conversions() {
    let io_err = io::Error::new(io::ErrorKind::NotFound, "file not found");
    let barq_err = BarqError::from(io_err);
    assert!(format!("{}", barq_err).contains("file not found"));
    
    let json_err = serde_json::from_str::<serde_json::Value>("{ invalid }").unwrap_err();
    let barq_err = BarqError::from(json_err);
    assert!(format!("{}", barq_err).contains("expected") || format!("{}", barq_err).contains("key"));
}

#[test]
fn test_result_propagation() {
    fn produces_err() -> BarqResult<()> {
        Err(BarqError::InvalidInput("bad input".to_string()))
    }
    
    fn propagates() -> BarqResult<()> {
        produces_err()?;
        Ok(())
    }
    
    let res = propagates();
    assert!(res.is_err());
    if let Err(BarqError::InvalidInput(msg)) = res {
        assert_eq!(msg, "bad input");
    } else {
        panic!("Wrong error variant");
    }
}
