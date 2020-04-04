//! Change Endpoint implementation.

use crate::accounts::AccountInfo;
use crate::changes::*;
use crate::error::Error;
use crate::{GerritRestApi, Result};
use ::http::StatusCode;
use serde_derive::Serialize;
use std::collections::BTreeMap;

/// Implement trait [ChangeEndpoint](trait.ChangeEndpoint.html) for Gerrit REST API.
impl ChangeEndpoint for GerritRestApi {
    fn create_change(&mut self, change: &ChangeInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json("/a/changes/", change)?
            .expect(StatusCode::CREATED)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    fn query_changes(&mut self, query: &QueryParams) -> Result<Vec<Vec<ChangeInfo>>> {
        let params = serde_url_params::to_string(query)?;
        let url = format!(
            "/a/changes/{}{}",
            if params.is_empty() { "" } else { "?" },
            params
        );
        let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
        let changes =
            if query.search_queries.is_some() && query.search_queries.as_ref().unwrap().len() > 1 {
                serde_json::from_str::<Vec<Vec<ChangeInfo>>>(&json)?
            } else {
                vec![serde_json::from_str::<Vec<ChangeInfo>>(&json)?]
            };
        Ok(changes)
    }

    fn get_change(
        &mut self, change_id: &str, additional_opts: Option<Vec<AdditionalOpt>>,
    ) -> Result<ChangeInfo> {
        let query = QueryParams {
            search_queries: None,
            additional_opts,
            limit: None,
            start: None,
        };
        let params = serde_url_params::to_string(&query)?;
        let url = format!(
            "/a/changes/{}/{}{}",
            change_id,
            if params.is_empty() { "" } else { "?" },
            params
        );
        let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    fn get_change_detail(
        &mut self, change_id: &str, additional_opts: Option<Vec<AdditionalOpt>>,
    ) -> Result<ChangeInfo> {
        let query = QueryParams {
            search_queries: None,
            additional_opts,
            limit: None,
            start: None,
        };
        let params = serde_url_params::to_string(&query)?;
        let url = format!(
            "/a/changes/{}/detail/{}{}",
            change_id,
            if params.is_empty() { "" } else { "?" },
            params
        );
        let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    fn create_merge_patch_set(
        &mut self, change_id: &str, input: &MergePatchSetInput,
    ) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(format!("/a/changes/{}/merge", change_id).as_str(), input)?
            .expect(StatusCode::OK)?
            .json()?;
        let change: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change)
    }

    fn set_commit_message(
        &mut self, change_id: &str, input: &CommitMessageInput,
    ) -> Result<ChangeInfo> {
        let json = self
            .rest
            .put_json(format!("/a/changes/{}/message", change_id).as_str(), input)?
            .expect(StatusCode::OK)?
            .json()?;
        let change: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change)
    }

    fn delete_change(&mut self, change_id: &str) -> Result<()> {
        self.rest
            .delete(format!("/a/changes/{}", change_id).as_str())?
            .expect(StatusCode::NO_CONTENT)?;
        Ok(())
    }

    fn get_topic(&mut self, change_id: &str) -> Result<String> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/topic", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let topic: String = serde_json::from_str(&json)?;
        Ok(topic)
    }

    fn set_topic(&mut self, change_id: &str, topic: &TopicInput) -> Result<String> {
        let json = self
            .rest
            .put_json(format!("/a/changes/{}/topic", change_id).as_str(), topic)?
            .expect(StatusCode::OK)?
            .json()?;
        let topic: String = serde_json::from_str(&json)?;
        Ok(topic)
    }

    fn delete_topic(&mut self, change_id: &str) -> Result<()> {
        self.rest
            .delete(format!("/a/changes/{}/topic", change_id).as_str())?
            .expect(StatusCode::NO_CONTENT)?;
        Ok(())
    }

    fn get_assignee(&mut self, change_id: &str) -> Result<AccountInfo> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/assignee", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let assignee: AccountInfo = serde_json::from_str(&json)?;
        Ok(assignee)
    }

    fn get_past_assignees(&mut self, change_id: &str) -> Result<Vec<AccountInfo>> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/past_assignees", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let past_assignees: Vec<AccountInfo> = serde_json::from_str(&json)?;
        Ok(past_assignees)
    }

    fn set_assignee(&mut self, change_id: &str, assignee: &AssigneeInput) -> Result<AccountInfo> {
        let json = self
            .rest
            .put_json(
                format!("/a/changes/{}/assignee", change_id).as_str(),
                assignee,
            )?
            .expect(StatusCode::OK)?
            .json()?;
        let assignee: AccountInfo = serde_json::from_str(&json)?;
        Ok(assignee)
    }

    fn delete_assignee(&mut self, change_id: &str) -> Result<AccountInfo> {
        let json = self
            .rest
            .delete(format!("/a/changes/{}/assignee", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let assignee: AccountInfo = serde_json::from_str(&json)?;
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
            "/a/changes/{}/pure_revert{}{}",
            change_id,
            if params.is_empty() { "" } else { "?" },
            params
        );
        let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
        let pure_revert: PureRevertInfo = serde_json::from_str(&json)?;
        Ok(pure_revert)
    }

    fn abandon_change(&mut self, change_id: &str, abandon: &AbandonInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(
                format!("/a/changes/{}/abandon", change_id).as_str(),
                abandon,
            )?
            .expect(StatusCode::OK)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    fn restore_change(&mut self, change_id: &str, restore: &RestoreInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(
                format!("/a/changes/{}/restore", change_id).as_str(),
                restore,
            )?
            .expect(StatusCode::OK)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    fn rebase_change(&mut self, change_id: &str, rebase: &RebaseInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(format!("/a/changes/{}/rebase", change_id).as_str(), rebase)?
            .expect(StatusCode::OK)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    fn move_change(&mut self, change_id: &str, move_input: &MoveInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(
                format!("/a/changes/{}/move", change_id).as_str(),
                move_input,
            )?
            .expect(StatusCode::OK)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    fn revert_change(&mut self, change_id: &str, revert: &RevertInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(format!("/a/changes/{}/revert", change_id).as_str(), revert)?
            .expect(StatusCode::OK)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    fn revert_submission(
        &mut self, change_id: &str, revert: &RevertInput,
    ) -> Result<RevertSubmissionInfo> {
        let json = self
            .rest
            .post_json(
                format!("/a/changes/{}/revert_submission", change_id).as_str(),
                revert,
            )?
            .expect(StatusCode::OK)?
            .json()?;
        let revert_submission: RevertSubmissionInfo = serde_json::from_str(&json)?;
        Ok(revert_submission)
    }

    fn submit_change(&mut self, change_id: &str, submit: &SubmitInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(format!("/a/changes/{}/submit", change_id).as_str(), submit)?
            .expect(StatusCode::OK)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
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
            "/a/changes/{}/submitted_together?o=NON_VISIBLE_CHANGES{}{}",
            change_id,
            if params.is_empty() { "" } else { "&" },
            params
        );
        let json = self.rest.get(&url)?.expect(StatusCode::OK)?.json()?;
        let submitted_together: SubmittedTogetherInfo = serde_json::from_str(&json)?;
        Ok(submitted_together)
    }

    fn list_change_comments(&mut self, change_id: &str) -> Result<BTreeMap<String, CommentInfo>> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/comments", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let comments: BTreeMap<String, CommentInfo> = serde_json::from_str(&json)?;
        Ok(comments)
    }

    fn list_change_drafts(&mut self, change_id: &str) -> Result<BTreeMap<String, CommentInfo>> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/drafts", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let drafts: BTreeMap<String, CommentInfo> = serde_json::from_str(&json)?;
        Ok(drafts)
    }

    fn list_change_messages(&mut self, change_id: &str) -> Result<Vec<ChangeMessageInfo>> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/messages", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let messages: Vec<ChangeMessageInfo> = serde_json::from_str(&json)?;
        Ok(messages)
    }

    fn get_change_message(
        &mut self, change_id: &str, message_id: &str,
    ) -> Result<ChangeMessageInfo> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/messages/{}", change_id, message_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let message: ChangeMessageInfo = serde_json::from_str(&json)?;
        Ok(message)
    }

    fn delete_change_message(
        &mut self, change_id: &str, message_id: &str, input: Option<&DeleteChangeMessageInput>,
    ) -> Result<ChangeMessageInfo> {
        let json = if let Some(input) = input {
            self.rest
                .post_json(
                    format!("/a/changes/{}/messages/{}/delete", change_id, message_id).as_str(),
                    input,
                )?
                .expect(StatusCode::OK)?
                .json()?
        } else {
            self.rest
                .delete(format!("/a/changes/{}/messages/{}", change_id, message_id).as_str())?
                .expect(StatusCode::OK)?
                .json()?
        };
        let message: ChangeMessageInfo = serde_json::from_str(&json)?;
        Ok(message)
    }

    fn list_reviewers(&mut self, change_id: &str) -> Result<Vec<ReviewerInfo>> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/reviewers/", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let reviewers: Vec<ReviewerInfo> = serde_json::from_str(&json)?;
        Ok(reviewers)
    }

    fn get_reviewer(&mut self, change_id: &str, account_id: &str) -> Result<ReviewerInfo> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/reviewers/{}", change_id, account_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let reviewer: ReviewerInfo = serde_json::from_str(&json)?;
        Ok(reviewer)
    }

    fn add_reviewer(
        &mut self, change_id: &str, reviewer: &ReviewerInput,
    ) -> Result<AddReviewerResult> {
        let json = self
            .rest
            .post_json(
                format!("/a/changes/{}/reviewers/", change_id).as_str(),
                reviewer,
            )?
            .expect(StatusCode::OK)?
            .json()?;
        let result: AddReviewerResult = serde_json::from_str(&json)?;
        Ok(result)
    }

    fn delete_reviewer(
        &mut self, change_id: &str, account_id: &str, input: Option<&DeleteReviewerInput>,
    ) -> Result<()> {
        if let Some(input) = input {
            self.rest
                .post_json(
                    format!("/a/changes/{}/reviewers/{}/delete", change_id, account_id).as_str(),
                    input,
                )?
                .expect(StatusCode::NO_CONTENT)?
        } else {
            self.rest
                .delete(format!("/a/changes/{}/reviewers/{}", change_id, account_id).as_str())?
                .expect(StatusCode::NO_CONTENT)?
        };
        Ok(())
    }
}
