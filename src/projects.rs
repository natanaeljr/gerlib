//! Projects related REST endpoints.
//!
//! See [ProjectEndpoints](trait.ProjectEndpoints.html) trait for the REST API.

use crate::changes::WebLinkInfo;
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// REST API
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// This trait describes the project related REST endpoints.
pub trait ProjectEndpoints {}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// JSON Entities
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Display, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ProjectStatus {
  Active,
  ReadOnly,
  Hidden,
}

#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
/// The ProjectInfo entity contains information about a project.
pub struct ProjectInfo {
  /// The URL encoded project name.
  pub id: String,
  /// The name of the project.
  /// Not set if returned in a map where the project name is used as map key.
  pub name: Option<String>,
  /// The name of the parent project.
  /// ?-<n> if the parent project is not visible (<n> is a number which is increased for each non-visible project).
  pub parent: Option<String>,
  /// The description of the project.
  pub description: Option<String>,
  /// The state of the project.
  pub state: Option<ProjectStatus>,
  /// Map of branch names to HEAD revisions.
  pub branches: Option<HashMap<String, String>>,
  /// Map of label names to LabelTypeInfo entries. This field is filled for Create Project and Get Project calls.
  pub labels: Option<HashMap<String, LabelTypeInfo>>,
  /// Links to the project in external sites as a list of WebLinkInfo entries.
  pub web_links: Option<Vec<WebLinkInfo>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct LabelTypeInfo {}
