use serde::Serialize;

#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct SupportReportResponse {
    pub file_name: String,
    pub issue_opened: bool,
    pub finder_revealed: bool,
}
