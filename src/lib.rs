#![allow(dead_code)]

extern crate strum;
#[macro_use]
extern crate strum_macros;

use crate::changes::*;
use crate::handler::RestHandler;
use crate::http::HttpRequestHandler;
use ::http::StatusCode;
use url::Url;

use crate::accounts::AccountInfo;
pub use crate::http::AuthMethod as HttpAuthMethod;
use std::collections::BTreeMap;

pub mod accounts;
pub mod changes;
pub mod details;
pub mod error;
pub mod projects;

mod handler;
mod http;

type Result<T> = std::result::Result<T, crate::error::Error>;

/// Gerrit REST API over HTTP.
///
/// The API is suitable for automated tools to build upon, as well as supporting some ad-hoc scripting use cases.
pub struct GerritRestApi {
    rest: RestHandler,
}

impl GerritRestApi {
    /// Create a new GerritRestApi with the host url, username and HTTP password.
    ///
    /// Additional configuration is available through specific methods below.
    pub fn new(base_url: Url, username: &str, password: &str) -> Result<Self> {
        let http = HttpRequestHandler::new(base_url, username, password)?;
        let rest = RestHandler::new(http);
        Ok(Self { rest })
    }

    /// Specify the HTTP authentication method.
    pub fn http_auth(mut self, auth: &HttpAuthMethod) -> Result<Self> {
        self.rest = RestHandler::new(self.rest.http().http_auth(auth)?);
        Ok(self)
    }

    /// Enable/Disable SSL verification of both host and peer.
    pub fn ssl_verify(mut self, enable: bool) -> Result<Self> {
        self.rest = RestHandler::new(self.rest.http().ssl_verify(enable)?);
        Ok(self)
    }

    /// Create a new change.
    ///
    /// The change input ChangeInput entity must be provided.
    ///
    /// To create a change the calling user must be allowed to upload to code review.
    ///
    /// As response a ChangeInfo entity is returned that describes the resulting change.
    pub fn create_change(&mut self, change: &ChangeInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json("/a/changes/", change)?
            .expect(StatusCode::CREATED)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    /// Queries changes visible to the caller.
    ///
    /// The query string must be provided by the q parameter. The n parameter can be used to limit
    /// the returned results. The no-limit parameter can be used remove the default limit on queries
    /// and return all results. This might not be supported by all index backends.
    ///
    /// As result a list of ChangeInfo entries is returned. The change output is sorted by the last
    /// update time, most recently updated to oldest updated.
    ///
    /// If the number of changes matching the query exceeds either the internal limit or
    /// a supplied n query parameter, the last change object has a _more_changes: true JSON field set.
    /// The S or start query parameter can be supplied to skip a number of changes from the list.
    /// Clients are allowed to specify more than one query by setting the q parameter multiple times.
    /// In this case the result is an array of arrays, one per query in the same order the queries were given in.
    pub fn query_changes(&mut self, query: &QueryParams) -> Result<Vec<Vec<ChangeInfo>>> {
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

    /// Retrieves a change.
    ///
    /// Additional fields can be obtained by adding o parameters, each option requires more database
    /// lookups and slows down the query response time to the client so they are generally disabled
    /// by default. Fields are described in Query Changes.
    ///
    /// As response a ChangeInfo entity is returned that describes the change.
    pub fn get_change(
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

    /// Retrieves a change with labels, detailed labels, detailed accounts, reviewer updates, and messages.
    ///
    /// Additional fields can be obtained by adding o parameters, each option requires more database
    /// lookups and slows down the query response time to the client so they are generally disabled
    /// by default. Fields are described in Query Changes.
    ///
    /// As response a ChangeInfo entity is returned that describes the change.
    /// This response will contain all votes for each label and include one combined vote.
    /// The combined label vote is calculated in the following order (from highest to lowest):
    /// REJECTED > APPROVED > DISLIKED > RECOMMENDED.
    pub fn get_change_detail(
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

    /// Deletes a change.
    ///
    /// New or abandoned changes can be deleted by their owner if the user is granted the
    /// `Delete Own Changes` permission, otherwise only by administrators.
    pub fn delete_change(&mut self, change_id: &str) -> Result<()> {
        self.rest
            .delete(format!("/a/changes/{}", change_id).as_str())?
            .expect(StatusCode::NO_CONTENT)?;
        Ok(())
    }

    /// Retrieves the topic of a change.
    ///
    /// If the change does not have a topic an empty string is returned.
    pub fn get_topic(&mut self, change_id: &str) -> Result<String> {
        let json = &self
            .rest
            .get(format!("/a/changes/{}/topic", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let topic: String = serde_json::from_str(&json)?;
        Ok(topic)
    }

    /// Sets the topic of a change.
    ///
    /// The new topic must be provided in the request body inside a TopicInput entity.
    /// Any leading or trailing whitespace in the topic name will be removed.
    ///
    /// As response the new topic is returned.
    pub fn set_topic(&mut self, change_id: &str, topic: &TopicInput) -> Result<String> {
        let json = &self
            .rest
            .put_json(format!("/a/changes/{}/topic", change_id).as_str(), topic)?
            .expect(StatusCode::OK)?
            .json()?;
        let topic: String = serde_json::from_str(&json)?;
        Ok(topic)
    }

    /// Deletes the topic of a change.
    pub fn delete_topic(&mut self, change_id: &str) -> Result<()> {
        self.rest
            .delete(format!("/a/changes/{}/topic", change_id).as_str())?
            .expect(StatusCode::NO_CONTENT)?;
        Ok(())
    }

    /// Retrieves the account of the user assigned to a change.
    ///
    /// As a response an AccountInfo entity describing the assigned account is returned.
    pub fn get_assignee(&mut self, change_id: &str) -> Result<AccountInfo> {
        let json = &self
            .rest
            .get(format!("/a/changes/{}/assignee", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let assignee: AccountInfo = serde_json::from_str(&json)?;
        Ok(assignee)
    }

    /// Returns a list of every user ever assigned to a change, in the order in which they were first assigned.
    ///
    /// [NOTE] Past assignees are only available when NoteDb is enabled.
    ///
    /// As a response a list of AccountInfo entities is returned.
    pub fn get_past_assignees(&mut self, change_id: &str) -> Result<Vec<AccountInfo>> {
        let json = &self
            .rest
            .get(format!("/a/changes/{}/past_assignees", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let past_assignees: Vec<AccountInfo> = serde_json::from_str(&json)?;
        Ok(past_assignees)
    }

    /// Sets the assignee of a change.
    ///
    /// The new assignee must be provided in the request body inside a AssigneeInput entity.
    ///
    /// As a response an AccountInfo entity describing the assigned account is returned.
    pub fn set_assignee(
        &mut self, change_id: &str, assignee: &AssigneeInput,
    ) -> Result<AccountInfo> {
        let json = &self
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

    /// Deletes the assignee of a change.
    ///
    /// As a response an AccountInfo entity describing the account of the deleted assignee is returned.
    ///
    /// If the change had no assignee the response is “204 No Content”.
    pub fn delete_assignee(&mut self, change_id: &str) -> Result<AccountInfo> {
        let json = &self
            .rest
            .delete(format!("/a/changes/{}/assignee", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let assignee: AccountInfo = serde_json::from_str(&json)?;
        Ok(assignee)
    }

    /// Abandons a change.
    ///
    /// The request body does not need to include a AbandonInput entity if no review comment is added.
    ///
    /// As response a ChangeInfo entity is returned that describes the abandoned change.
    ///
    /// If the change cannot be abandoned because the change state doesn’t allow abandoning of the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    ///
    /// An email will be sent using the "abandon" template. The notify handling is ALL.
    /// Notifications are suppressed on WIP changes that have never started review.
    pub fn abandon_change(
        &mut self, change_id: &str, abandon: &AbandonInput,
    ) -> Result<ChangeInfo> {
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

    /// Restores a change.
    ///
    /// The request body does not need to include a RestoreInput entity if no review comment is added.
    ///
    /// As response a ChangeInfo entity is returned that describes the restored change.
    ///
    /// If the change cannot be restored because the change state doesn't allow restoring the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    pub fn restore_change(
        &mut self, change_id: &str, restore: &RestoreInput,
    ) -> Result<ChangeInfo> {
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

    /// Rebases a change.
    ///
    /// Optionally, the parent revision can be changed to another patch set through the RebaseInput entity.
    ///
    /// As response a ChangeInfo entity is returned that describes the rebased change.
    /// Information about the current patch set is included.
    ///
    /// If the change cannot be rebased, e.g. due to conflicts, the response is “409 Conflict” and
    /// the error message is contained in the response body.
    pub fn rebase_change(&mut self, change_id: &str, rebase: &RebaseInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(format!("/a/changes/{}/rebase", change_id).as_str(), rebase)?
            .expect(StatusCode::OK)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    /// Move a change.
    ///
    /// The destination branch must be provided in the request body inside a MoveInput entity.
    ///
    /// As response a ChangeInfo entity is returned that describes the moved change.
    ///
    /// Note that this endpoint will not update the change’s parents, which is different from the cherry-pick endpoint.
    ///
    /// If the change cannot be moved because the change state doesn't allow moving the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    ///
    /// If the change cannot be moved because the user doesn't have abandon permission on the change
    /// or upload permission on the destination, the response is “409 Conflict” and the error message
    /// is contained in the response body.
    pub fn move_change(&mut self, change_id: &str, move_input: &MoveInput) -> Result<ChangeInfo> {
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

    /// Reverts a change.
    ///
    /// The subject of the newly created change will be 'Revert "<subject-of-reverted-change>"'.
    /// If the subject of the change reverted is above 63 characters, it will be cut down to 59 characters with "…​" in the end.
    ///
    /// The request body does not need to include a RevertInput entity if no review comment is added.
    ///
    /// As response a ChangeInfo entity is returned that describes the reverting change.
    ///
    /// If the change cannot be reverted because the change state doesn’t allow reverting the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    pub fn revert_change(&mut self, change_id: &str, revert: &RevertInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(format!("/a/changes/{}/revert", change_id).as_str(), revert)?
            .expect(StatusCode::OK)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    /// Submits a change.
    ///
    /// The request body only needs to include a SubmitInput entity if submitting on behalf of another user.
    ///
    /// As response a ChangeInfo entity is returned that describes the submitted/merged change.
    ///
    /// If the change cannot be submitted because the submit rule doesn’t allow submitting the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    pub fn submit_change(&mut self, change_id: &str, submit: &SubmitInput) -> Result<ChangeInfo> {
        let json = self
            .rest
            .post_json(format!("/a/changes/{}/submit", change_id).as_str(), submit)?
            .expect(StatusCode::OK)?
            .json()?;
        let change_info: ChangeInfo = serde_json::from_str(&json)?;
        Ok(change_info)
    }

    /// Lists the published comments of all revisions of the change.
    ///
    /// Returns a map of file paths to lists of CommentInfo entries. The entries in the map are
    /// sorted by file path, and the comments for each path are sorted by patch set number.
    /// Each comment has the patch_set and author fields set.
    pub fn list_change_comments(
        &mut self, change_id: &str,
    ) -> Result<BTreeMap<String, CommentInfo>> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/comments", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let comments: BTreeMap<String, CommentInfo> = serde_json::from_str(&json)?;
        Ok(comments)
    }

    /// Lists the draft comments of all revisions of the change that belong to the calling user.
    ///
    /// Returns a map of file paths to lists of CommentInfo entries.
    /// The entries in the map are sorted by file path, and the comments for each path are sorted by
    /// patch set number. Each comment has the `patch_set` field set, and no `author`.
    pub fn list_change_drafts(&mut self, change_id: &str) -> Result<BTreeMap<String, CommentInfo>> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/drafts", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let drafts: BTreeMap<String, CommentInfo> = serde_json::from_str(&json)?;
        Ok(drafts)
    }

    /// Lists all the messages of a change including detailed account information.
    ///
    /// As response a list of ChangeMessageInfo entities is returned.
    pub fn list_change_messages(&mut self, change_id: &str) -> Result<Vec<ChangeMessageInfo>> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/messages", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let messages: Vec<ChangeMessageInfo> = serde_json::from_str(&json)?;
        Ok(messages)
    }

    /// Retrieves a change message including detailed account information.
    ///
    /// As response a ChangeMessageInfo entity is returned.
    pub fn get_change_message(
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

    /// Deletes a change message by replacing the change message with a new message, which contains
    /// the name of the user who deleted the change message and the reason why it was deleted.
    /// The reason can be provided in the request body as a DeleteChangeMessageInput entity.
    ///
    /// Note that only users with the Administrate Server global capability are permitted to delete
    /// a change message.
    ///
    /// As response a ChangeMessageInfo entity is returned that describes the updated change message.
    pub fn delete_change_message(
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

    /// Lists the reviewers of a change.
    ///
    /// As result a list of ReviewerInfo entries is returned.
    pub fn list_reviewers(&mut self, change_id: &str) -> Result<Vec<ReviewerInfo>> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/reviewers/", change_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let reviewers: Vec<ReviewerInfo> = serde_json::from_str(&json)?;
        Ok(reviewers)
    }

    /// Retrieves a reviewer of a change.
    ///
    /// As response a ReviewerInfo entity is returned that describes the reviewer.
    pub fn get_reviewer(&mut self, change_id: &str, account_id: &str) -> Result<ReviewerInfo> {
        let json = self
            .rest
            .get(format!("/a/changes/{}/reviewers/{}", change_id, account_id).as_str())?
            .expect(StatusCode::OK)?
            .json()?;
        let reviewer: ReviewerInfo = serde_json::from_str(&json)?;
        Ok(reviewer)
    }

    /// Adds one user or all members of one group as reviewer to the change.
    ///
    /// The reviewer to be added to the change must be provided in the request body as a ReviewerInput entity.
    ///
    /// Users can be moved from reviewer to CC and vice versa. This means if a user is added as CC that is
    /// already a reviewer on the change, the reviewer state of that user is updated to CC.
    /// If a user that is already a CC on the change is added as reviewer, the reviewer state of that user
    /// is updated to reviewer.
    pub fn add_reviewer(
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

    /// Adds one user or all members of one group as reviewer to the change.
    ///
    /// The reviewer to be added to the change must be provided in the request body as a ReviewerInput entity.
    ///
    /// Users can be moved from reviewer to CC and vice versa. This means if a user is added as CC that is
    /// already a reviewer on the change, the reviewer state of that user is updated to CC.
    /// If a user that is already a CC on the change is added as reviewer, the reviewer state of that user
    /// is updated to reviewer.
    pub fn delete_reviewer(
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
