use std::collections::{BTreeMap, HashMap};
use std::fs::File;
use std::io::BufWriter;
use std::path::Path;

use mcap::records::MessageHeader;
use mcap::write::Writer;

use clankers_core::{RobotError, RobotResult, Timestamp};

/// MCAP file writer for recording node I/O.
pub struct McapWriter {
    writer: Writer<BufWriter<File>>,
    channels: HashMap<String, u16>,
}

impl McapWriter {
    pub fn create(path: impl AsRef<Path>) -> RobotResult<Self> {
        let file = File::create(path.as_ref())
            .map_err(|e| RobotError::Data(format!("create mcap: {e}")))?;
        let writer =
            Writer::new(BufWriter::new(file)).map_err(|e| RobotError::Data(e.to_string()))?;
        Ok(Self {
            writer,
            channels: HashMap::new(),
        })
    }

    pub fn write_message(
        &mut self,
        topic: &str,
        schema_name: &str,
        encoding: &str,
        data: &[u8],
        log_time: Timestamp,
    ) -> RobotResult<()> {
        let channel_id = self.get_or_create_channel(topic, schema_name, encoding, data)?;

        self.writer
            .write_to_known_channel(
                &MessageHeader {
                    channel_id,
                    sequence: 0,
                    log_time: log_time.as_nanos(),
                    publish_time: log_time.as_nanos(),
                },
                data,
            )
            .map_err(|e| RobotError::Data(e.to_string()))?;
        Ok(())
    }

    pub fn finish(mut self) -> RobotResult<()> {
        self.writer
            .finish()
            .map_err(|e| RobotError::Data(e.to_string()))?;
        Ok(())
    }

    fn get_or_create_channel(
        &mut self,
        topic: &str,
        schema_name: &str,
        encoding: &str,
        schema_data: &[u8],
    ) -> RobotResult<u16> {
        if let Some(&id) = self.channels.get(topic) {
            return Ok(id);
        }
        let schema_id = self
            .writer
            .add_schema(schema_name, encoding, schema_data)
            .map_err(|e| RobotError::Data(e.to_string()))?;
        let channel_id = self
            .writer
            .add_channel(schema_id, topic, encoding, &BTreeMap::new())
            .map_err(|e| RobotError::Data(e.to_string()))?;
        self.channels.insert(topic.to_string(), channel_id);
        Ok(channel_id)
    }
}
