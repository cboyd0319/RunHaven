use runhaven_core::ui_contracts::{ManagedSessionListData, ManagedSessionSummaryData};
use serde::ser::Error as _;
use serde::{Serialize, Serializer};

#[derive(Clone, Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct ManagedSessionListResponse {
    pub sessions: Vec<ManagedSessionSummaryResponse>,
}

#[derive(Clone, Debug)]
pub(crate) struct ManagedSessionSummaryResponse {
    pub session: ManagedSessionSummaryData,
    pub output_available: bool,
}

impl ManagedSessionListResponse {
    pub(crate) fn from_data(
        data: ManagedSessionListData,
        output_available: impl Fn(&str) -> Result<bool, String>,
    ) -> Result<Self, String> {
        let sessions = data
            .sessions
            .into_iter()
            .map(|session| {
                let available = output_available(&session.run_id)?;
                Ok(ManagedSessionSummaryResponse {
                    session,
                    output_available: available,
                })
            })
            .collect::<Result<Vec<_>, String>>()?;
        Ok(Self { sessions })
    }
}

impl Serialize for ManagedSessionSummaryResponse {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut value = serde_json::to_value(&self.session).map_err(S::Error::custom)?;
        let object = value.as_object_mut().ok_or_else(|| {
            S::Error::custom("managed session summary did not serialize as an object")
        })?;
        object.insert(
            "outputAvailable".to_string(),
            serde_json::Value::Bool(self.output_available),
        );
        value.serialize(serializer)
    }
}
