//! Change Endpoint implementation.

use crate::accounts::AccountInfo;
use crate::changes::*;
use crate::{GerritRestApi, Result};
use ::http::StatusCode;
use serde_derive::Serialize;
use serde_with::skip_serializing_none;
use std::collections::BTreeMap;

/// Implement trait [ChangeEndpoints](trait.ChangeEndpoints.html) for Gerrit REST API.
impl ChangeEndpoints for GerritRestApi {
  fn create_change(&mut self, change: &ChangeInput) -> Result<ChangeInfo> {
    let json = self
      .rest
      .post_json("a/changes/", change)?
      .expect(StatusCode::CREATED)?
      .json()?;
    let change_info = serde_json::from_str(&json)?;
    Ok(change_info)
  }

  fn query_changes(&mut self, query: &QueryParams) -> Result<Vec<Vec<ChangeInfo>>> {
    let params = serde_url_params::to_string(query)?;
    let url = format!("a/changes/{}{}", if params.is_empty() { "" } else { "?" }, params);
    let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
    let changes = if query.search_queries.is_some() && query.search_queries.as_ref().unwrap().len() > 1 {
      serde_json::from_str::<Vec<Vec<ChangeInfo>>>(&json)?
    } else {
      vec![serde_json::from_str::<Vec<ChangeInfo>>(&json)?]
    };
    Ok(changes)
  }

  fn get_change(&mut self, change_id: &str, additional_opts: Option<Vec<AdditionalOpt>>) -> Result<ChangeInfo> {
    let query = QueryParams {
      search_queries: None,
      additional_opts,
      limit: None,
      start: None,
    };
    let params = serde_url_params::to_string(&query)?;
    let url = format!(
      "a/changes/{}/{}{}",
      change_id,
      if params.is_empty() { "" } else { "?" },
      params
    );
    let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
    let change_info = serde_json::from_str(&json)?;
    Ok(change_info)
  }

  fn get_change_detail(&mut self, change_id: &str, additional_opts: Option<Vec<AdditionalOpt>>) -> Result<ChangeInfo> {
    let query = QueryParams {
      search_queries: None,
      additional_opts,
      limit: None,
      start: None,
    };
    let params = serde_url_params::to_string(&query)?;
    let url = format!(
      "a/changes/{}/detail/{}{}",
      change_id,
      if params.is_empty() { "" } else { "?" },
      params
    );
    let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
    let change_info = serde_json::from_str(&json)?;
    Ok(change_info)
  }

  fn create_merge_patch_set(&mut self, change_id: &str, input: &MergePatchSetInput) -> Result<ChangeInfo> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/merge", change_id).as_str(), input)?
      .expect(StatusCode::OK)?
      .json()?;
    let change = serde_json::from_str(&json)?;
    Ok(change)
  }

  fn set_commit_message(&mut self, change_id: &str, input: &CommitMessageInput) -> Result<ChangeInfo> {
    let json = self
      .rest
      .put_json(format!("a/changes/{}/message", change_id).as_str(), input)?
      .expect(StatusCode::OK)?
      .json()?;
    let change = serde_json::from_str(&json)?;
    Ok(change)
  }

  fn delete_change(&mut self, change_id: &str) -> Result<()> {
    self
      .rest
      .delete(format!("a/changes/{}", change_id).as_str())?
      .expect(StatusCode::NO_CONTENT)?;
    Ok(())
  }

  fn get_topic(&mut self, change_id: &str) -> Result<String> {
    let json = self
      .rest
      .get(format!("a/changes/{}/topic", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let topic = serde_json::from_str(&json)?;
    Ok(topic)
  }

  fn set_topic(&mut self, change_id: &str, topic: &TopicInput) -> Result<String> {
    let json = self
      .rest
      .put_json(format!("a/changes/{}/topic", change_id).as_str(), topic)?
      .expect(StatusCode::OK)?
      .json()?;
    let topic = serde_json::from_str(&json)?;
    Ok(topic)
  }

  fn delete_topic(&mut self, change_id: &str) -> Result<()> {
    self
      .rest
      .delete(format!("a/changes/{}/topic", change_id).as_str())?
      .expect(StatusCode::NO_CONTENT)?;
    Ok(())
  }

  fn get_assignee(&mut self, change_id: &str) -> Result<AccountInfo> {
    let json = self
      .rest
      .get(format!("a/changes/{}/assignee", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let assignee = serde_json::from_str(&json)?;
    Ok(assignee)
  }

  fn get_past_assignees(&mut self, change_id: &str) -> Result<Vec<AccountInfo>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/past_assignees", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let past_assignees = serde_json::from_str(&json)?;
    Ok(past_assignees)
  }

  fn set_assignee(&mut self, change_id: &str, assignee: &AssigneeInput) -> Result<AccountInfo> {
    let json = self
      .rest
      .put_json(format!("a/changes/{}/assignee", change_id).as_str(), assignee)?
      .expect(StatusCode::OK)?
      .json()?;
    let assignee = serde_json::from_str(&json)?;
    Ok(assignee)
  }

  fn delete_assignee(&mut self, change_id: &str) -> Result<AccountInfo> {
    let json = self
      .rest
      .delete(format!("a/changes/{}/assignee", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let assignee = serde_json::from_str(&json)?;
    Ok(assignee)
  }

  fn get_pure_revert(&mut self, change_id: &str, commit: Option<&str>) -> Result<PureRevertInfo> {
    #[derive(Serialize)]
    pub struct Query<'a> {
      #[serde(rename = "o", skip_serializing_if = "Option::is_none")]
      pub option: Option<&'a str>,
    }
    let query = Query { option: commit };
    let params = serde_url_params::to_string(&query)?;
    let url = format!(
      "a/changes/{}/pure_revert{}{}",
      change_id,
      if params.is_empty() { "" } else { "?" },
      params
    );
    let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
    let pure_revert = serde_json::from_str(&json)?;
    Ok(pure_revert)
  }

  fn abandon_change(&mut self, change_id: &str, abandon: &AbandonInput) -> Result<ChangeInfo> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/abandon", change_id).as_str(), abandon)?
      .expect(StatusCode::OK)?
      .json()?;
    let change_info = serde_json::from_str(&json)?;
    Ok(change_info)
  }

  fn restore_change(&mut self, change_id: &str, restore: &RestoreInput) -> Result<ChangeInfo> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/restore", change_id).as_str(), restore)?
      .expect(StatusCode::OK)?
      .json()?;
    let change_info = serde_json::from_str(&json)?;
    Ok(change_info)
  }

  fn rebase_change(&mut self, change_id: &str, rebase: &RebaseInput) -> Result<ChangeInfo> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/rebase", change_id).as_str(), rebase)?
      .expect(StatusCode::OK)?
      .json()?;
    let change_info = serde_json::from_str(&json)?;
    Ok(change_info)
  }

  fn move_change(&mut self, change_id: &str, move_input: &MoveInput) -> Result<ChangeInfo> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/move", change_id).as_str(), move_input)?
      .expect(StatusCode::OK)?
      .json()?;
    let change_info = serde_json::from_str(&json)?;
    Ok(change_info)
  }

  fn revert_change(&mut self, change_id: &str, revert: &RevertInput) -> Result<ChangeInfo> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/revert", change_id).as_str(), revert)?
      .expect(StatusCode::OK)?
      .json()?;
    let change_info = serde_json::from_str(&json)?;
    Ok(change_info)
  }

  fn revert_submission(&mut self, change_id: &str, revert: &RevertInput) -> Result<RevertSubmissionInfo> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/revert_submission", change_id).as_str(), revert)?
      .expect(StatusCode::OK)?
      .json()?;
    let revert_submission = serde_json::from_str(&json)?;
    Ok(revert_submission)
  }

  fn submit_change(&mut self, change_id: &str, submit: &SubmitInput) -> Result<ChangeInfo> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/submit", change_id).as_str(), submit)?
      .expect(StatusCode::OK)?
      .json()?;
    let change_info = serde_json::from_str(&json)?;
    Ok(change_info)
  }

  fn changes_submitted_together(
    &mut self, change_id: &str, additional_opts: Option<&Vec<AdditionalOpt>>,
  ) -> Result<SubmittedTogetherInfo> {
    #[derive(Serialize)]
    pub struct Query<'a> {
      #[serde(rename = "o", skip_serializing_if = "Option::is_none")]
      pub additional_opts: Option<&'a Vec<AdditionalOpt>>,
    }
    let query = Query { additional_opts };
    let params = serde_url_params::to_string(&query)?;
    let url = format!(
      "a/changes/{}/submitted_together?o=NON_VISIBLE_CHANGES{}{}",
      change_id,
      if params.is_empty() { "" } else { "&" },
      params
    );
    let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
    let submitted_together = serde_json::from_str(&json)?;
    Ok(submitted_together)
  }

  fn get_included_in(&mut self, change_id: &str) -> Result<IncludedInInfo> {
    let json = self
      .rest
      .get(format!("a/changes/{}/in", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let included_in = serde_json::from_str(&json)?;
    Ok(included_in)
  }

  fn index_change(&mut self, change_id: &str) -> Result<()> {
    self
      .rest
      .post(format!("a/changes/{}/index", change_id).as_str())?
      .expect(StatusCode::NO_CONTENT)?;
    Ok(())
  }

  fn list_change_comments(&mut self, change_id: &str) -> Result<BTreeMap<String, CommentInfo>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/comments", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let comments = serde_json::from_str(&json)?;
    Ok(comments)
  }

  fn list_change_robot_comments(&mut self, change_id: &str) -> Result<BTreeMap<String, RobotCommentInfo>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/robotcomments", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let robot_comments = serde_json::from_str(&json)?;
    Ok(robot_comments)
  }

  fn list_change_drafts(&mut self, change_id: &str) -> Result<BTreeMap<String, CommentInfo>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/drafts", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let drafts = serde_json::from_str(&json)?;
    Ok(drafts)
  }

  fn check_change(&mut self, change_id: &str) -> Result<ChangeInfo> {
    let json = self
      .rest
      .get(format!("a/changes/{}/check", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let changes = serde_json::from_str(&json)?;
    Ok(changes)
  }

  fn fix_change(&mut self, change_id: &str) -> Result<ChangeInfo> {
    let json = self
      .rest
      .post(format!("a/changes/{}/check", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let changes = serde_json::from_str(&json)?;
    Ok(changes)
  }

  fn set_work_in_progress(&mut self, change_id: &str, input: Option<&WorkInProgressInput>) -> Result<()> {
    let url = format!("a/changes/{}/wip", change_id);
    if let Some(input) = input {
      self.rest.post_json(&url, input)?
    } else {
      self.rest.post(&url)?
    }
    .expect(StatusCode::OK)?;
    Ok(())
  }

  fn set_ready_for_review(&mut self, change_id: &str, input: Option<&WorkInProgressInput>) -> Result<()> {
    let url = format!("a/changes/{}/ready", change_id);
    if let Some(input) = input {
      self.rest.post_json(&url, input)?
    } else {
      self.rest.post(&url)?
    }
    .expect(StatusCode::OK)?;
    Ok(())
  }

  fn mark_private(&mut self, change_id: &str, input: Option<&PrivateInput>) -> Result<()> {
    let url = format!("a/changes/{}/private", change_id);
    if let Some(input) = input {
      self.rest.post_json(&url, input)?
    } else {
      self.rest.post(&url)?
    }
    .expect_or(StatusCode::CREATED)?
    .expect(StatusCode::OK)?;
    Ok(())
  }

  fn unmark_private(&mut self, change_id: &str, input: Option<&PrivateInput>) -> Result<()> {
    if let Some(input) = input {
      self
        .rest
        .post_json(format!("a/changes/{}/private.delete", change_id).as_str(), input)?
    } else {
      self.rest.delete(format!("a/changes/{}/private", change_id).as_str())?
    }
    .expect(StatusCode::NO_CONTENT)?;
    Ok(())
  }

  fn ignore_change(&mut self, change_id: &str) -> Result<()> {
    self
      .rest
      .put(format!("a/changes/{}/ignore", change_id).as_str())?
      .expect(StatusCode::OK)?;
    Ok(())
  }

  fn unignore_change(&mut self, change_id: &str) -> Result<()> {
    self
      .rest
      .put(format!("a/changes/{}/unignore", change_id).as_str())?
      .expect(StatusCode::OK)?;
    Ok(())
  }

  fn mark_as_reviewed(&mut self, change_id: &str) -> Result<()> {
    self
      .rest
      .put(format!("a/changes/{}/reviewed", change_id).as_str())?
      .expect(StatusCode::OK)?;
    Ok(())
  }

  fn mark_as_unreviewed(&mut self, change_id: &str) -> Result<()> {
    self
      .rest
      .put(format!("a/changes/{}/unreviewed", change_id).as_str())?
      .expect(StatusCode::OK)?;
    Ok(())
  }

  fn get_hashtags(&mut self, change_id: &str) -> Result<Vec<String>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/hashtags", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let hashtags = serde_json::from_str(&json)?;
    Ok(hashtags)
  }

  fn set_hashtags(&mut self, change_id: &str, input: &HashtagsInput) -> Result<Vec<String>> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/hashtags", change_id).as_str(), input)?
      .expect(StatusCode::OK)?
      .json()?;
    let hashtags = serde_json::from_str(&json)?;
    Ok(hashtags)
  }

  fn list_change_messages(&mut self, change_id: &str) -> Result<Vec<ChangeMessageInfo>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/messages", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let messages = serde_json::from_str(&json)?;
    Ok(messages)
  }

  fn get_change_message(&mut self, change_id: &str, message_id: &str) -> Result<ChangeMessageInfo> {
    let json = self
      .rest
      .get(format!("a/changes/{}/messages/{}", change_id, message_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let message = serde_json::from_str(&json)?;
    Ok(message)
  }

  fn delete_change_message(
    &mut self, change_id: &str, message_id: &str, input: Option<&DeleteChangeMessageInput>,
  ) -> Result<ChangeMessageInfo> {
    let json = if let Some(input) = input {
      self
        .rest
        .post_json(
          format!("a/changes/{}/messages/{}/delete", change_id, message_id).as_str(),
          input,
        )?
        .expect(StatusCode::OK)?
        .json()?
    } else {
      self
        .rest
        .delete(format!("a/changes/{}/messages/{}", change_id, message_id).as_str())?
        .expect(StatusCode::OK)?
        .json()?
    };
    let message = serde_json::from_str(&json)?;
    Ok(message)
  }

  fn list_reviewers(&mut self, change_id: &str) -> Result<Vec<ReviewerInfo>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/reviewers/", change_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let reviewers = serde_json::from_str(&json)?;
    Ok(reviewers)
  }

  fn suggest_reviewers(
    &mut self, change_id: &str, query_str: &str, limit: Option<u32>, exclude_groups: bool, cc: bool,
  ) -> Result<Vec<SuggestedReviewerInfo>> {
    #[skip_serializing_none]
    #[derive(Serialize)]
    pub struct Query<'a> {
      #[serde(rename = "q")]
      pub query_str: &'a str,
      #[serde(rename = "n")]
      pub limit: Option<u32>,
      #[serde(rename = "exclude-groups")]
      pub exclude_groups: Option<()>,
      #[serde(rename = "reviewer-state")]
      pub reviewer_state: Option<&'static str>,
    }
    let query = Query {
      query_str,
      limit,
      exclude_groups: if exclude_groups { Some(()) } else { None },
      reviewer_state: if cc { Some("CC") } else { None },
    };
    let params = serde_url_params::to_string(&query)?;
    let url = format!(
      "a/changes/{}/suggest_reviewers{}{}",
      change_id,
      if params.is_empty() { "" } else { "?" },
      params
    );
    let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
    let reviewers = serde_json::from_str(&json)?;
    Ok(reviewers)
  }

  fn get_reviewer(&mut self, change_id: &str, account_id: &str) -> Result<ReviewerInfo> {
    let json = self
      .rest
      .get(format!("a/changes/{}/reviewers/{}", change_id, account_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let reviewer = serde_json::from_str(&json)?;
    Ok(reviewer)
  }

  fn add_reviewer(&mut self, change_id: &str, reviewer: &ReviewerInput) -> Result<AddReviewerResult> {
    let json = self
      .rest
      .post_json(format!("a/changes/{}/reviewers/", change_id).as_str(), reviewer)?
      .expect(StatusCode::OK)?
      .json()?;
    let result = serde_json::from_str(&json)?;
    Ok(result)
  }

  fn delete_reviewer(&mut self, change_id: &str, account_id: &str, input: Option<&DeleteReviewerInput>) -> Result<()> {
    if let Some(input) = input {
      self
        .rest
        .post_json(
          format!("a/changes/{}/reviewers/{}/delete", change_id, account_id).as_str(),
          input,
        )?
        .expect(StatusCode::NO_CONTENT)?
    } else {
      self
        .rest
        .delete(format!("a/changes/{}/reviewers/{}", change_id, account_id).as_str())?
        .expect(StatusCode::NO_CONTENT)?
    };
    Ok(())
  }

  fn list_votes(&mut self, change_id: &str, account_id: &str) -> Result<BTreeMap<String, i32>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/reviewers/{}/votes/", change_id, account_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let votes = serde_json::from_str(&json)?;
    Ok(votes)
  }

  fn delete_vote(
    &mut self, change_id: &str, account_id: &str, label_id: &str, input: Option<&DeleteVoteInput>,
  ) -> Result<()> {
    let url = format!("a/changes/{}/reviewers/{}/votes/{}", change_id, account_id, label_id);
    if let Some(input) = input {
      self.rest.post_json(format!("{}/delete", url).as_str(), input)?
    } else {
      self.rest.delete(&url)?
    }
    .expect(StatusCode::NO_CONTENT)?;
    Ok(())
  }

  fn get_commit(&mut self, change_id: &str, revision_id: &str, links: bool) -> Result<CommitInfo> {
    #[skip_serializing_none]
    #[derive(Serialize)]
    pub struct Query {
      pub links: Option<()>,
    }
    let query = Query {
      links: if links { Some(()) } else { None },
    };
    let params = serde_url_params::to_string(&query)?;
    let url = format!(
      "a/changes/{}/revisions/{}/commit{}{}",
      change_id,
      revision_id,
      if params.is_empty() { "" } else { "?" },
      params
    );

    let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
    let commit = serde_json::from_str(&json)?;
    Ok(commit)
  }

  fn get_description(&mut self, change_id: &str, revision_id: &str) -> Result<String> {
    let json = self
      .rest
      .get(format!("a/changes/{}/revisions/{}/description", change_id, revision_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let description = serde_json::from_str(&json)?;
    Ok(description)
  }

  fn set_description(&mut self, change_id: &str, revision_id: &str, input: &DescriptionInput) -> Result<String> {
    let json = self
      .rest
      .put_json(
        format!("a/changes/{}/revisions/{}/description", change_id, revision_id).as_str(),
        input,
      )?
      .expect(StatusCode::OK)?
      .json()?;
    let description = serde_json::from_str(&json)?;
    Ok(description)
  }

  fn get_merge_list(&mut self, change_id: &str, revision_id: &str) -> Result<Vec<CommitInfo>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/revisions/{}/mergelist", change_id, revision_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let commits = serde_json::from_str(&json)?;
    Ok(commits)
  }

  fn get_revision_actions(&mut self, change_id: &str, revision_id: &str) -> Result<BTreeMap<String, ActionInfo>> {
    let json = self
      .rest
      .get(format!("a/changes/{}/revisions/{}/actions", change_id, revision_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let actions = serde_json::from_str(&json)?;
    Ok(actions)
  }

  fn get_review(&mut self, change_id: &str, revision_id: &str) -> Result<ChangeInfo> {
    let json = self
      .rest
      .get(format!("a/changes/{}/revisions/{}/review", change_id, revision_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let change = serde_json::from_str(&json)?;
    Ok(change)
  }

  fn set_review(&mut self, change_id: &str, revision_id: &str, input: &ReviewInput) -> Result<ReviewResult> {
    let json = self
      .rest
      .post_json(
        format!("a/changes/{}/revisions/{}/review", change_id, revision_id).as_str(),
        input,
      )?
      .expect(StatusCode::OK)?
      .json()?;
    let result = serde_json::from_str(&json)?;
    Ok(result)
  }

  fn get_related_changes(&mut self, change_id: &str, revision_id: &str) -> Result<RelatedChangesInfo> {
    let json = self
      .rest
      .get(format!("a/changes/{}/revisions/{}/related", change_id, revision_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let related = serde_json::from_str(&json)?;
    Ok(related)
  }

  fn rebase_revision(&mut self, change_id: &str, revision_id: &str, input: Option<&RebaseInput>) -> Result<ChangeInfo> {
    let url = format!("a/changes/{}/revisions/{}/rebase", change_id, revision_id);
    let json = if let Some(input) = input {
      self.rest.post_json(&url, input)?
    } else {
      self.rest.post(&url)?
    }
    .expect(StatusCode::OK)?
    .json()?;
    let change = serde_json::from_str(&json)?;
    Ok(change)
  }

  fn submit_revision(&mut self, change_id: &str, revision_id: &str) -> Result<SubmitInfo> {
    let json = self
      .rest
      .post(format!("a/changes/{}/revisions/{}/submit", change_id, revision_id).as_str())?
      .expect(StatusCode::OK)?
      .json()?;
    let submit = serde_json::from_str(&json)?;
    Ok(submit)
  }

  fn get_patch(&mut self, change_id: &str, revision_id: &str, opts: &Option<PatchParams>) -> Result<Vec<u8>> {
    let params = if let Some(opts) = opts {
      serde_url_params::to_string(opts)?
    } else {
      String::default()
    };
    let url = format!(
      "a/changes/{}/revisions/{}/patch{}{}",
      change_id,
      revision_id,
      if params.is_empty() { "" } else { "?" },
      params
    );
    let patch = self.rest.get(&url)?.expect(StatusCode::OK)?.raw();
    Ok(patch)
  }

  fn submit_preview(&mut self, change_id: &str, revision_id: &str, format: CompressFormat) -> Result<Vec<u8>> {
    todo!()
  }

  fn list_drafts(&mut self, change_id: &str, revision_id: &str) -> Result<BTreeMap<String, CommentInfo>> {
    todo!()
  }

  fn create_draft(&mut self, change_id: &str, revision_id: &str, input: &CommentInput) -> Result<CommentInfo> {
    todo!()
  }

  fn get_draft(&mut self, change_id: &str, revision_id: &str, draft_id: &str) -> Result<CommentInfo> {
    todo!()
  }

  fn update_draft(&mut self, change_id: &str, revision_id: &str, input: &CommentInput) -> Result<CommentInfo> {
    todo!()
  }

  fn delete_draft(&mut self, change_id: &str, revision_id: &str, draft_id: &str) -> Result<()> {
    todo!()
  }

  fn list_comments(&mut self, change_id: &str, revision_id: &str) -> Result<BTreeMap<String, Vec<CommentInfo>>> {
    todo!()
  }

  fn get_comment(&mut self, change_id: &str, revision_id: &str, comment_id: &str) -> Result<CommentInfo> {
    todo!()
  }

  fn delete_comment(&mut self, change_id: &str, revision_id: &str, comment_id: &str) -> Result<CommentInfo> {
    todo!()
  }

  fn list_files(
    &mut self, change_id: &str, revision_id: &str, opts: &Option<ListFilesParams>,
  ) -> Result<BTreeMap<String, FileInfo>> {
    todo!()
  }

  fn get_content(
    &mut self, change_id: &str, revision_id: &str, file_id: &str, opts: &Option<GetContentParams>,
  ) -> Result<Vec<u8>> {
    todo!()
  }

  fn get_diff(
    &mut self, change_id: &str, revision_id: &str, file_id: &str, opts: &Option<DiffParams>,
  ) -> Result<DiffInfo> {
    todo!()
  }
}
