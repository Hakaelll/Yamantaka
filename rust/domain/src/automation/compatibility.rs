use super::*;
use sha2::{Digest, Sha256};
use std::io::{self, Write};
struct HashWriter(Sha256);
impl Write for HashWriter {
    fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
        self.0.update(bytes);
        Ok(bytes.len())
    }
    fn flush(&mut self) -> io::Result<()> {
        Ok(())
    }
}
pub fn flutter_compatibility_hash(id: &str, definition: &RuleDefinition) -> Result<String> {
    #[derive(Serialize)]
    struct Payload<'a> {
        id: &'a str,
        trigger: &'a RuleTrigger,
        actions: &'a [RuleAction],
    }
    let mut writer = HashWriter(Sha256::new());
    serde_json::to_writer(
        &mut writer,
        &Payload {
            id,
            trigger: &definition.trigger,
            actions: &definition.actions,
        },
    )
    .map_err(|_| Failure::Invalid("Rule compatibility could not be checked".into()))?;
    Ok(format!("{:x}", writer.0.finalize()))
}
