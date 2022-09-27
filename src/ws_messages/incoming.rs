use super::serializer::IncomingSerializer as is;

use serde::{Deserialize, Serialize};
use tracing::debug;

#[derive(Debug)]
pub enum MessageValues {
    Valid(ParsedMessage),
    Invalid(ErrorData),
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "name", content = "body")]
pub enum ParsedMessage {
    AddAlarm(AddAlarm),
    DeleteAll,
    DeleteOne(DeleteOne),
    LedStatus,
    Light { status: bool },
    Restart,
    Status,
    TimeZone(TimeZone),
}

#[derive(Deserialize, Debug, Serialize)]
pub struct AddAlarm {
    #[serde(deserialize_with = "is::days")]
    pub days: Vec<u8>,
    #[serde(deserialize_with = "is::hour")]
    pub hour: u8,
    #[serde(deserialize_with = "is::minute")]
    pub minute: u8,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct DeleteOne {
    #[serde(deserialize_with = "is::id")]
    pub alarm_id: i64,
}

#[derive(Deserialize, Debug, Serialize)]
pub struct TimeZone {
    #[serde(deserialize_with = "is::timezone")]
    pub zone: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case")]
struct StructuredMessage {
    data: Option<ParsedMessage>,
    error: Option<ErrorData>,
    unique: Option<String>,
}

// TODO
#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "snake_case", tag = "error", content = "message")]
pub enum ErrorData {
    Something(String),
}

pub fn to_struct(input: &str) -> Option<MessageValues> {
    let user_serialized = serde_json::from_str::<StructuredMessage>(input);
    if let Ok(data) = user_serialized {
        if let Some(data) = data.error {
            return Some(MessageValues::Invalid(data));
        }
        if let Some(data) = data.data {
            return Some(MessageValues::Valid(data));
        }
        None
    } else {
        let error_serialized = serde_json::from_str::<ErrorData>(input);
        if let Ok(data) = error_serialized {
            debug!("Matched error_serialized data");
            Some(MessageValues::Invalid(data))
        } else {
            debug!("not a known input message");
            None
        }
    }
}

/// message_incoming
///
/// cargo watch -q -c -w src/ -x 'test message_incoming -- --test-threads=1 --nocapture'
#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::too_many_lines)]
mod tests {
    use super::*;

    #[test]
    fn message_incoming_parse_invalid() {
        let data = r#""#;
        let result = to_struct(data);
        assert!(result.is_none());

        let data = r#"{}"#;
        let result = to_struct(data);
        assert!(result.is_none());
    }

    #[test]
    fn message_incoming_parse_add_alarm_valid() {
        let data = r#"
            {
            	"data": {
            		"name" : "add_alarm",
            		"body": {
            			"hour":6,"minute":15,"days":[0,1,2,3,4,5,6]
            		}
            	}
            }"#;
        let result = to_struct(data);
        assert!(result.is_some());
        let result = result.unwrap();
        match result {
            MessageValues::Valid(ParsedMessage::AddAlarm(data)) => {
                assert_eq!(data.days, vec![0, 1, 2, 3, 4, 5, 6]);
                assert_eq!(data.hour, 6);
                assert_eq!(data.minute, 15);
            }
            _ => unreachable!("Shouldn't have matched this"),
        };
    }

    #[test]
    fn message_incoming_parse_add_alarm_invalid() {
        // No body
        let data = r#"
    {
    	"data": {
    		"name" : "add_alarm",
    	}
    }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // Empty body
        let data = r#"
    		{
    			"data": {
    				"name" : "add_alarm",
    				"body: "",
    			}
    		}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // Empty body object
        let data = r#"
    		{
    			"data": {
    				"name" : "add_alarm",
    				"body: {},
    			}
    		}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // No hours
        let data = r#"
    		  {
    			  "data": {
    				  "name" : "add_alarm",
    				  "body: {"minute":6,"days":[0,1,2,3,4,5,6]},
    			  }
    		  }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid hours - number as string
        let data = r#"
    		  {
    			  "data": {
    				  "name" : "add_alarm",
    				  "body: {"hour":"6","minute":4, "days":[0,1,2,3,4,5,6]},
    			  }
    		  }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid hours - string
        let data = r#"
    			{
    				"data": {
    					"name" : "add_alarm",
    					"body: {"hour":"string","minute":4, "days":[0,1,2,3,4,5,6]},
    				}
    			}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // No minute
        let data = r#"
    		  {
    			  "data": {
    				  "name" : "add_alarm",
    				  "body: {"hour":6,"days":[0,1,2,3,4,5,6]},
    			  }
    		  }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid minute - number as string
        let data = r#"
    		  {
    			  "data": {
    				  "name" : "add_alarm",
    				  "body: {"hour":6,"minutes":"4", "days":[0,1,2,3,4,5,6]},
    			  }
    		  }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid minutes - string
        let data = r#"
    			{
    				"data": {
    					"name" : "add_alarm",
    					"body: {"hour":6,"minutes":"string", "days":[0,1,2,3,4,5,6]},
    				}
    			}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid minutes - > 59
        let data = r#"
    			{
    				"data": {
    					"name" : "add_alarm",
    					"body: {"hour":6,"minutes":"61", "days":[0,1,2,3,4,5,6]},
    				}
    			}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // No days
        let data = r#"
    		  {
    			  "data": {
    				  "name" : "add_alarm",
    				  "body: {"hour":6,"minute":3},
    			  }
    		  }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid days - number
        let data = r#"
    		  {
    			  "data": {
    				  "name" : "add_alarm",
    				  "body: {"hour":6,"minute":4, "days":"1"},
    			  }
    		  }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid days - string
        let data = r#"
    			{
    				"data": {
    					"name" : "add_alarm",
    					"body: {"hour":6,"minute":1, "days":"string"},
    				}
    			}"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid days - vec of number strings
        let data = r#"
    		 {
    			 "data": {
    				 "name" : "add_alarm",
    				 "body: {"hour":6,"minute":4, "days":["1", "2"]},
    			 }
    		 }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid days - vec of number strings
        let data = r#"
		   {
			   "data": {
				   "name" : "add_alarm",
				   "body: {"hour":6,"minute":4, "days":[8]},
			   }
		   }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid days - vec of string strings
        let data = r#"
    		 {
    			 "data": {
    				 "name" : "add_alarm",
    				 "body: {"hour":6,"minute":4, "days":["one", "two"]},
    			 }
    		 }"#;
        let result = to_struct(data);
        assert!(result.is_none());

        // invalid days - day > 6
        let data = r#"
			{
				"data": {
					"name" : "add_alarm",
					"body: {"hour":6,"minute":4, "days":[7]},
				}
			}"#;
        let result = to_struct(data);
        assert!(result.is_none());
    }
}
