//! Change Endpoint module.
//!
//! See [ChangeEndpoint](trait.ChangeEndpoint.html) trait for the REST API.

use crate::accounts::{AccountInfo, AccountInput, GpgKeyInfo};
use crate::details::Timestamp;
use crate::Result;
use serde::{Serialize, Serializer};
use serde_derive::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::{BTreeMap, HashMap};
use std::fmt::{Display, Error, Formatter};

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// REST API
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// This trait describes the change related REST endpoints.
pub trait ChangeEndpoint {
    /// Create a new change.
    ///
    /// The change input `ChangeInput` entity must be provided.
    ///
    /// To create a change the calling user must be allowed to upload to code review.
    ///
    /// As response a `ChangeInfo` entity is returned that describes the resulting change.
    fn create_change(&mut self, change: &ChangeInput) -> Result<ChangeInfo>;

    /// Queries changes visible to the caller.
    ///
    /// The query string must be provided by the q parameter. The n parameter can be used to limit
    /// the returned results. The no-limit parameter can be used remove the default limit on queries
    /// and return all results. This might not be supported by all index backends.
    ///
    /// As result a list of `ChangeInfo` entries is returned. The change output is sorted by the last
    /// update time, most recently updated to oldest updated.
    ///
    /// If the number of changes matching the query exceeds either the internal limit or
    /// a supplied n query parameter, the last change object has a `_more_changes: true` JSON field set.
    /// The S or start query parameter can be supplied to skip a number of changes from the list.
    /// Clients are allowed to specify more than one query by setting the q parameter multiple times.
    /// In this case the result is an array of arrays, one per query in the same order the queries were given in.
    fn query_changes(&mut self, query: &QueryParams) -> Result<Vec<Vec<ChangeInfo>>>;

    /// Retrieves a change.
    ///
    /// Additional fields can be obtained by adding o parameters, each option requires more database
    /// lookups and slows down the query response time to the client so they are generally disabled
    /// by default. Fields are described in Query Changes.
    ///
    /// As response a `ChangeInfo` entity is returned that describes the change.
    fn get_change(
        &mut self, change_id: &str, additional_opts: Option<Vec<AdditionalOpt>>,
    ) -> Result<ChangeInfo>;

    /// Retrieves a change with labels, detailed labels, detailed accounts, reviewer updates, and messages.
    ///
    /// Additional fields can be obtained by adding o parameters, each option requires more database
    /// lookups and slows down the query response time to the client so they are generally disabled
    /// by default. Fields are described in Query Changes.
    ///
    /// As response a `ChangeInfo` entity is returned that describes the change.
    /// This response will contain all votes for each label and include one combined vote.
    /// The combined label vote is calculated in the following order (from highest to lowest):
    /// REJECTED > APPROVED > DISLIKED > RECOMMENDED.
    fn get_change_detail(
        &mut self, change_id: &str, additional_opts: Option<Vec<AdditionalOpt>>,
    ) -> Result<ChangeInfo>;

    /// Update an existing change by using a `MergePatchSetInput` entity.
    ///
    /// Gerrit will create a merge commit based on the information of `MergePatchSetInput` and add
    /// a new patch set to the change corresponding to the new merge commit.
    ///
    /// As response a `ChangeInfo` entity with current revision is returned that describes the resulting change.
    fn create_merge_patch_set(
        &mut self, change_id: &str, input: &MergePatchSetInput,
    ) -> Result<ChangeInfo>;

    /// Creates a new patch set with a new commit message.
    ///
    /// The new commit message must be provided in the request body inside a `CommitMessageInput` entity.
    /// If a Change-Id footer is specified, it must match the current Change-Id footer.
    /// If the Change-Id footer is absent, the current Change-Id is added to the message.
    fn set_commit_message(
        &mut self, change_id: &str, input: &CommitMessageInput,
    ) -> Result<ChangeInfo>;

    /// Deletes a change.
    ///
    /// New or abandoned changes can be deleted by their owner if the user is granted the
    /// `Delete Own Changes` permission, otherwise only by administrators.
    fn delete_change(&mut self, change_id: &str) -> Result<()>;

    /// Retrieves the topic of a change.
    ///
    /// If the change does not have a topic an empty string is returned.
    fn get_topic(&mut self, change_id: &str) -> Result<String>;

    /// Sets the topic of a change.
    ///
    /// The new topic must be provided in the request body inside a `TopicInput` entity.
    /// Any leading or trailing whitespace in the topic name will be removed.
    ///
    /// As response the new topic is returned.
    fn set_topic(&mut self, change_id: &str, topic: &TopicInput) -> Result<String>;

    /// Deletes the topic of a change.
    fn delete_topic(&mut self, change_id: &str) -> Result<()>;

    /// Retrieves the account of the user assigned to a change.
    ///
    /// As a response an `AccountInfo` entity describing the assigned account is returned.
    fn get_assignee(&mut self, change_id: &str) -> Result<AccountInfo>;

    /// Returns a list of every user ever assigned to a change, in the order in which they were first assigned.
    ///
    /// NOTE: Past assignees are only available when NoteDb is enabled.
    ///
    /// As a response a list of `AccountInfo` entities is returned.
    fn get_past_assignees(&mut self, change_id: &str) -> Result<Vec<AccountInfo>>;

    /// Sets the assignee of a change.
    ///
    /// The new assignee must be provided in the request body inside a `AssigneeInput` entity.
    ///
    /// As a response an `AccountInfo` entity describing the assigned account is returned.
    fn set_assignee(&mut self, change_id: &str, assignee: &AssigneeInput) -> Result<AccountInfo>;

    /// Deletes the assignee of a change.
    ///
    /// As a response an `AccountInfo` entity describing the account of the deleted assignee is returned.
    ///
    /// If the change had no assignee the response is “204 No Content”.
    fn delete_assignee(&mut self, change_id: &str) -> Result<AccountInfo>;

    /// Check if the given change is a pure revert of the change it references in revertOf.
    ///
    /// Optionally, the query parameter `o` can be passed in to specify a commit (SHA1 in 40 digit hex representation)
    /// to check against. It takes precedence over revertOf. If the change has no reference in revertOf,
    /// the parameter is mandatory.
    ///
    /// As response a `PureRevertInfo` entity is returned.
    fn get_pure_revert(&mut self, change_id: &str, commit: Option<&str>) -> Result<PureRevertInfo>;

    /// Abandons a change.
    ///
    /// The request body does not need to include a `AbandonInput` entity if no review comment is added.
    ///
    /// As response a `ChangeInfo` entity is returned that describes the abandoned change.
    ///
    /// If the change cannot be abandoned because the change state doesn’t allow abandoning of the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    ///
    /// An email will be sent using the "abandon" template. The notify handling is ALL.
    /// Notifications are suppressed on WIP changes that have never started review.
    fn abandon_change(&mut self, change_id: &str, abandon: &AbandonInput) -> Result<ChangeInfo>;

    /// Restores a change.
    ///
    /// The request body does not need to include a `RestoreInput` entity if no review comment is added.
    ///
    /// As response a `ChangeInfo` entity is returned that describes the restored change.
    ///
    /// If the change cannot be restored because the change state doesn't allow restoring the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    fn restore_change(&mut self, change_id: &str, restore: &RestoreInput) -> Result<ChangeInfo>;

    /// Rebases a change.
    ///
    /// Optionally, the parent revision can be changed to another patch set through the `RebaseInput` entity.
    ///
    /// As response a `ChangeInfo` entity is returned that describes the rebased change.
    /// Information about the current patch set is included.
    ///
    /// If the change cannot be rebased, e.g. due to conflicts, the response is “409 Conflict” and
    /// the error message is contained in the response body.
    fn rebase_change(&mut self, change_id: &str, rebase: &RebaseInput) -> Result<ChangeInfo>;

    /// Move a change.
    ///
    /// The destination branch must be provided in the request body inside a `MoveInput` entity.
    ///
    /// As response a `ChangeInfo` entity is returned that describes the moved change.
    ///
    /// Note that this endpoint will not update the change’s parents, which is different from the cherry-pick endpoint.
    ///
    /// If the change cannot be moved because the change state doesn't allow moving the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    ///
    /// If the change cannot be moved because the user doesn't have abandon permission on the change
    /// or upload permission on the destination, the response is “409 Conflict” and the error message
    /// is contained in the response body.
    fn move_change(&mut self, change_id: &str, move_input: &MoveInput) -> Result<ChangeInfo>;

    /// Reverts a change.
    ///
    /// The subject of the newly created change will be 'Revert "<subject-of-reverted-change>"'.
    /// If the subject of the change reverted is above 63 characters, it will be cut down to 59 characters with "…​" in the end.
    ///
    /// The request body does not need to include a `RevertInput` entity if no review comment is added.
    ///
    /// As response a `ChangeInfo` entity is returned that describes the reverting change.
    ///
    /// If the change cannot be reverted because the change state doesn’t allow reverting the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    fn revert_change(&mut self, change_id: &str, revert: &RevertInput) -> Result<ChangeInfo>;

    /// Creates open revert changes for all of the changes of a certain submission.
    ///
    /// The subject of each revert change will be 'Revert "<subject-of-reverted-change"'.
    /// If the subject is above 60 characters, the subject will be cut to 56 characters with "…​" in the end.
    /// However, whenever reverting the submission of a revert submission, the subject will be shortened
    /// from 'Revert "Revert "<subject-of-reverted-change""' to 'Revert^2 "<subject-of-reverted-change"'.
    /// Also, for every future revert submission, the only difference in the subject will be the number of
    /// the revert (instead of Revert^2 the subject will change to Revert^3 and so on).
    /// There are no guarantees about the subjects if the users change the default subjects.
    ///
    /// Details for the revert can be specified in the request body inside a `RevertInput`.
    /// The topic of all created revert changes will be `revert-{submission_id}-{random_string_of_size_10}`.
    ///
    /// The changes will not be rebased on onto the destination branch so the users may still have to
    /// manually rebase them to resolve conflicts and make them submittable.
    ///
    /// However, the changes that have the same project and branch will be rebased on top of each other.
    /// E.g, the first revert change will have the original change as a parent, and the second revert change
    /// will have the first revert change as a parent.
    ///
    /// There is one special case that involves merge commits; if a user has multiple changes in the
    /// same project and branch, but not in the same change series, those changes can still get submitted
    /// together if they have the same topic and `change.submitWholeTopic` in gerrit.config is set to true.
    /// In the case, Gerrit may create merge commits on submit (depending on the submit types of the project).
    /// The first parent for the reverts will be the most recent merge commit that was created by Gerrit to
    /// merge the different change series into the target branch.
    ///
    /// As response `RevertSubmissionInfo` entity is returned. That entity describes the revert changes.
    fn revert_submission(
        &mut self, change_id: &str, revert: &RevertInput,
    ) -> Result<RevertSubmissionInfo>;

    /// Submits a change.
    ///
    /// The request body only needs to include a `SubmitInput` entity if submitting on behalf of another user.
    ///
    /// As response a `ChangeInfo` entity is returned that describes the submitted/merged change.
    ///
    /// If the change cannot be submitted because the submit rule doesn’t allow submitting the change,
    /// the response is “409 Conflict” and the error message is contained in the response body.
    fn submit_change(&mut self, change_id: &str, submit: &SubmitInput) -> Result<ChangeInfo>;

    /// Computes list of all changes which are submitted when Submit is called for this change,
    /// including the current change itself.
    ///
    /// The list consists of:
    ///  * The given change.
    ///  * If `change.submitWholeTopic` is enabled, include all open changes with the same topic.
    ///  * For each change whose submit type is not CHERRY_PICK, include unmerged ancestors targeting the same branch.
    ///
    /// As a special case, the list is empty if this change would be submitted by itself (without other changes).
    ///
    /// As a response a `SubmittedTogetherInfo` entity is returned that describes what would happen
    /// if the change were submitted. This response contains a list of changes and a count of changes
    /// that are not visible to the caller that are part of the set of changes to be merged.
    ///
    /// The listed changes use the same format as in Query Changes with the LABELS, DETAILED_LABELS,
    /// CURRENT_REVISION, and SUBMITTABLE options set.
    fn changes_submitted_together(
        &mut self, change_id: &str, additional_opts: Option<&Vec<AdditionalOpt>>,
    ) -> Result<SubmittedTogetherInfo>;

    /// Retrieves the branches and tags in which a change is included.
    ///
    /// As result an `IncludedInInfo` entity is returned.
    fn get_included_in(&mut self, change_id: &str) -> Result<IncludedInInfo>;

    /// Adds or updates the change in the secondary index.
    fn index_change(&mut self, change_id: &str) -> Result<()>;

    /// Lists the published comments of all revisions of the change.
    ///
    /// Returns a map of file paths to lists of `CommentInfo` entries. The entries in the map are
    /// sorted by file path, and the comments for each path are sorted by patch set number.
    /// Each comment has the patch_set and author fields set.
    fn list_change_comments(&mut self, change_id: &str) -> Result<BTreeMap<String, CommentInfo>>;

    /// Lists the robot comments of all revisions of the change.
    ///
    /// Return a map that maps the file path to a list of RobotCommentInfo entries.
    /// The entries in the map are sorted by file path.
    fn list_change_robot_comments(
        &mut self, change_id: &str,
    ) -> Result<BTreeMap<String, RobotCommentInfo>>;

    /// Lists the draft comments of all revisions of the change that belong to the calling user.
    ///
    /// Returns a map of file paths to lists of `CommentInfo` entries.
    /// The entries in the map are sorted by file path, and the comments for each path are sorted by
    /// patch set number. Each comment has the `patch_set` field set, and no `author`.
    fn list_change_drafts(&mut self, change_id: &str) -> Result<BTreeMap<String, CommentInfo>>;

    /// Performs consistency checks on the change, and returns a ChangeInfo entity with the problems field
    /// set to a list of ProblemInfo entities.
    ///
    /// Depending on the type of problem, some fields not marked optional may be missing from the result.
    /// At least `id`, `project`, `branch`, and `_number` will be present.
    fn check_change(&mut self, change_id: &str) -> Result<ChangeInfo>;

    /// Performs consistency checks on the change as with `check_change`, and additionally fixes any
    /// problems that can be fixed automatically. The returned field values reflect any fixes.
    ///
    /// Some fixes have options controlling their behavior, which can be set in the `FixInput` entity body.
    ///
    /// Only the change owner, a project owner, or an administrator may fix changes.
    fn fix_change(&mut self, change_id: &str) -> Result<ChangeInfo>;

    /// Marks the change as not ready for review yet.
    ///
    /// Changes may only be marked not ready by the owner, project owners or site administrators.
    ///
    /// The request body does not need to include a `WorkInProgressInput` entity if no review comment is added.
    /// Actions that create a new patch set in a WIP change default to notifying **OWNER** instead of **ALL**.
    fn set_work_in_progress(
        &mut self, change_id: &str, input: Option<&WorkInProgressInput>,
    ) -> Result<()>;

    /// Marks the change as ready for review (set WIP property to false).
    ///
    /// Changes may only be marked ready by the owner, project owners or site administrators.
    ///
    /// Activates notifications of reviewer. The request body does not need to include a `WorkInProgressInput`
    /// entity if no review comment is added.
    fn set_ready_for_review(
        &mut self, change_id: &str, input: Option<&WorkInProgressInput>,
    ) -> Result<()>;

    /// Marks the change to be private.
    ///
    /// Only open changes can be marked private.
    ///
    /// Changes may only be marked private by the owner or site administrators.
    ///
    /// A message can be specified in the request body inside a `PrivateInput` entity.
    fn mark_private(&mut self, change_id: &str, input: Option<&PrivateInput>) -> Result<()>;

    /// Marks the change to be non-private.
    ///
    /// Note users can only unmark own private changes.
    ///
    /// If the change was already not private, the response is “409 Conflict”.
    ///
    /// A message can be specified in the request body inside a PrivateInput entity.
    fn unmark_private(&mut self, change_id: &str, input: Option<&PrivateInput>) -> Result<()>;

    /// Marks a change as ignored.
    ///
    /// The change will not be shown in the incoming reviews dashboard, and email notifications will be suppressed.
    ///
    /// Ignoring a change does not cause the change’s "updated" timestamp to be modified, and the owner is not notified.
    fn ignore_change(&mut self, change_id: &str) -> Result<()>;

    /// Un-marks a change as ignored.
    fn unignore_change(&mut self, change_id: &str) -> Result<()>;

    /// Marks a change as reviewed.
    ///
    /// This allows users to "de-highlight" changes in their dashboard until a new patch set is uploaded.
    ///
    /// This differs from the ignore endpoint, which will mute emails and hide the change from dashboard
    /// completely until it is unignored again.
    fn mark_as_reviewed(&mut self, change_id: &str) -> Result<()>;

    /// Marks a change as unreviewed.
    ///
    /// This allows users to "highlight" changes in their dashboard
    fn mark_as_unreviewed(&mut self, change_id: &str) -> Result<()>;

    /// Gets the hashtags associated with a change.
    ///
    /// NOTE: Hashtags are only available when NoteDb is enabled.
    ///
    /// As response the change's hashtags are returned as a list of strings.
    fn get_hashtags(&mut self, change_id: &str) -> Result<Vec<String>>;

    /// Adds and/or removes hashtags from a change.
    ///
    /// NOTE: Hashtags are only available when NoteDb is enabled.
    ///
    /// The hashtags to add or remove must be provided in the request body inside a `HashtagsInput` entity.
    ///
    /// As response the change's hashtags are returned as a list of strings.
    fn set_hashtags(&mut self, change_id: &str, input: &HashtagsInput) -> Result<Vec<String>>;

    /// Lists all the messages of a change including detailed account information.
    ///
    /// As response a list of `ChangeMessageInfo` entities is returned.
    fn list_change_messages(&mut self, change_id: &str) -> Result<Vec<ChangeMessageInfo>>;

    /// Retrieves a change message including detailed account information.
    ///
    /// As response a `ChangeMessageInfo` entity is returned.
    fn get_change_message(
        &mut self, change_id: &str, message_id: &str,
    ) -> Result<ChangeMessageInfo>;

    /// Deletes a change message by replacing the change message with a new message, which contains
    /// the name of the user who deleted the change message and the reason why it was deleted.
    /// The reason can be provided in the request body as a `DeleteChangeMessageInput` entity.
    ///
    /// Note that only users with the Administrate Server global capability are permitted to delete
    /// a change message.
    ///
    /// As response a `ChangeMessageInfo` entity is returned that describes the updated change message.
    fn delete_change_message(
        &mut self, change_id: &str, message_id: &str, input: Option<&DeleteChangeMessageInput>,
    ) -> Result<ChangeMessageInfo>;

    /// Lists the reviewers of a change.
    ///
    /// As result a list of `ReviewerInfo` entries is returned.
    fn list_reviewers(&mut self, change_id: &str) -> Result<Vec<ReviewerInfo>>;

    /// Retrieves a reviewer of a change.
    ///
    /// As response a `ReviewerInfo` entity is returned that describes the reviewer.
    fn get_reviewer(&mut self, change_id: &str, account_id: &str) -> Result<ReviewerInfo>;

    /// Adds one user or all members of one group as reviewer to the change.
    ///
    /// The reviewer to be added to the change must be provided in the request body as a `ReviewerInput` entity.
    ///
    /// Users can be moved from reviewer to CC and vice versa. This means if a user is added as CC that is
    /// already a reviewer on the change, the reviewer state of that user is updated to CC.
    /// If a user that is already a CC on the change is added as reviewer, the reviewer state of that user
    /// is updated to reviewer.
    fn add_reviewer(
        &mut self, change_id: &str, reviewer: &ReviewerInput,
    ) -> Result<AddReviewerResult>;

    /// Adds one user or all members of one group as reviewer to the change.
    ///
    /// The reviewer to be added to the change must be provided in the request body as a `ReviewerInput` entity.
    ///
    /// Users can be moved from reviewer to CC and vice versa. This means if a user is added as CC that is
    /// already a reviewer on the change, the reviewer state of that user is updated to CC.
    /// If a user that is already a CC on the change is added as reviewer, the reviewer state of that user
    /// is updated to reviewer.
    fn delete_reviewer(
        &mut self, change_id: &str, account_id: &str, input: Option<&DeleteReviewerInput>,
    ) -> Result<()>;
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// JSON Entities
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// The AbandonInput entity contains information for abandoning a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct AbandonInput {
    /// Message to be added as review comment to the change when abandoning the change.
    pub message: Option<String>,
    /// Notify handling that defines to whom email notifications should be sent
    /// after the change is abandoned.
    /// If not set, the default is ALL.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
}

/// The ActionInfo entity describes a REST API call the client can make to manipulate a resource.
/// These are frequently implemented by plugins and may be discovered at runtime.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ActionInfo {
    /// HTTP method to use with the action.
    /// Most actions use POST, PUT or DELETE to cause state changes.
    pub method: Option<HttpMethod>,
    /// Short title to display to a user describing the action. In the Gerrit web interface the
    /// label is used as the text on the button presented in the UI.
    pub label: Option<String>,
    /// Longer text to display describing the action. In a web UI this should be the title attribute
    /// of the element, displaying when the user hovers the mouse.
    pub title: Option<String>,
    /// If true the action is permitted at this time and the caller is likely allowed to execute it.
    /// This may change if state is updated at the server or permissions are modified.
    #[serde(default)]
    pub enabled: bool,
}

/// The AddReviewerResult entity describes the result of adding a reviewer to a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct AddReviewerResult {
    /// Value of the reviewer field from ReviewerInput set while adding the reviewer.
    pub input: String,
    /// The newly added reviewers as a list of ReviewerInfo entities.
    pub reviewers: Option<Vec<ReviewerInfo>>,
    /// The newly CCed accounts as a list of ReviewerInfo entities. This field will only appear if
    /// the requested state for the reviewer was CC and NoteDb is enabled on the server.
    pub ccs: Option<Vec<ReviewerInfo>>,
    /// Error message explaining why the reviewer could not be added.
    /// If a group was specified in the input and an error is returned, it means that none of the
    /// members were added as reviewer.
    pub error: Option<String>,
    /// Whether adding the reviewer requires confirmation.
    #[serde(default)]
    pub confirm: bool,
}

/// The ApprovalInfo entity contains information about an approval from a user for a label on a change.
/// ApprovalInfo has the same fields as AccountInfo. In addition to the following fields:
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ApprovalInfo {
    /// The account information entity.
    #[serde(flatten)]
    pub account: AccountInfo,
    /// The vote that the user has given for the label. If present and zero, the user is permitted
    /// to vote on the label. If absent, the user is not permitted to vote on that label.
    pub value: Option<i32>,
    /// The VotingRangeInfo the user is authorized to vote on that label. If present, the user is
    /// permitted to vote on the label regarding the range values. If absent, the user is not
    /// permitted to vote on that label.
    pub permitted_voting_range: Option<VotingRangeInfo>,
    /// The time and date describing when the approval was made.
    pub date: Option<Timestamp>,
    /// Value of the tag field from ReviewInput set while posting the review. Votes/comments that
    /// contain tag with 'autogenerated:' prefix can be filtered out in the web UI. NOTE: To apply
    /// different tags on different votes/comments multiple invocations of the REST call are required.
    pub tag: Option<String>,
    /// If true, this vote was made after the change was submitted.
    #[serde(default)]
    pub post_submit: bool,
}

/// The AssigneeInput entity contains the identity of the user to be set as assignee.
#[derive(Debug, Serialize, Deserialize)]
pub struct AssigneeInput {
    /// The ID of one account that should be added as assignee.
    pub assignee: String,
}

/// The BlameInfo entity stores the commit metadata with the row coordinates where it applies.
#[derive(Debug, Serialize, Deserialize)]
pub struct BlameInfo {
    /// The author of the commit.
    pub author: String,
    /// The id of the commit.
    pub id: String,
    /// Commit time.
    pub time: String,
    /// The commit message.
    pub commit_msg: String,
    /// The blame row coordinates as RangeInfo entities.
    pub ranges: Vec<RangeInfo>,
}

/// The ChangeEditInput entity contains information for restoring a path within change edit.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeEditInput {
    /// Path to file to restore.
    pub restore_path: Option<String>,
    /// Old path to file to rename.
    pub old_path: Option<String>,
    /// New path to file to rename.
    pub new_path: Option<String>,
}

/// The ChangeEditMessageInput entity contains information for changing the commit message
/// within a change edit.
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeEditMessageInput {
    /// New commit message.
    pub message: String,
}

/// The ChangeInfo entity contains information about a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeInfo {
    /// The ID of the change in the format "'<project>~<branch>~<Change-Id>'",
    /// where 'project', 'branch' and 'Change-Id' are URL encoded.
    /// For 'branch' the refs/heads/ prefix is omitted.
    pub id: String,
    /// The name of the project.
    pub project: String,
    /// The name of the target branch. The refs/heads/ prefix is omitted.
    pub branch: String,
    /// The topic to which this change belongs.
    pub topic: Option<String>,
    /// The assignee of the change as an AccountInfo entity.
    pub assignee: Option<AccountInfo>,
    /// List of hashtags that are set on the change (only populated when NoteDb is enabled).
    pub hashtags: Option<Vec<String>>,
    /// The Change-Id of the change.
    pub change_id: String,
    /// The subject of the change (header line of the commit message).
    pub subject: String,
    /// The status of the change.
    pub status: ChangeStatus,
    /// The timestamp of when the change was created.
    pub created: Timestamp,
    /// The timestamp of when the change was last updated.
    pub updated: Timestamp,
    /// The timestamp of when the change was submitted.
    pub submitted: Option<Timestamp>,
    /// The user who submitted the change, as an AccountInfo entity.
    pub submitter: Option<AccountInfo>,
    /// Whether the calling user has starred this change with the default label.
    #[serde(default)]
    pub starred: bool,
    /// A list of star labels that are applied by the calling user to this change.
    /// The labels are lexicographically sorted.
    pub stars: Option<Vec<String>>,
    /// Whether the change was reviewed by the calling user. Only set if reviewed is requested.
    #[serde(default)]
    pub reviewed: bool,
    /// The submit type of the change. Not set for merged changes.
    pub submit_type: Option<SubmitType>,
    /// Whether the change is mergeable. Not set for merged changes, if the change has not yet
    /// been tested, or if the skip_mergeable option is set or when
    /// change.api.excludeMergeableInChangeInfo is set.
    pub mergeable: Option<bool>,
    /// Whether the change has been approved by the project submit rules. Only set if requested.
    pub submittable: Option<bool>,
    /// Number of inserted lines.
    pub insertions: u32,
    /// Number of deleted lines.
    pub deletions: u32,
    /// Total number of inline comments across all patch sets.
    /// Not set if the current change index doesn’t have the data.
    pub total_comment_count: Option<u32>,
    /// Number of unresolved inline comment threads across all patch sets.
    /// Not set if the current change index doesn’t have the data.
    pub unresolved_comment_count: Option<u32>,
    /// The legacy numeric ID of the change.
    #[serde(rename = "_number")]
    pub number: u32,
    /// The owner of the change as an AccountInfo entity.
    pub owner: AccountInfo,
    /// Actions the caller might be able to perform on this revision.
    /// The information is a map of view name to ActionInfo entities.
    pub actions: Option<HashMap<String, ActionInfo>>,
    /// List of the requirements to be met before this change can be submitted.
    pub requirements: Option<Vec<Requirement>>,
    /// The labels of the change as a map that maps the label names to LabelInfo entries.
    /// Only set if labels or detailed labels are requested.
    pub labels: Option<BTreeMap<String, LabelInfo>>,
    /// A map of the permitted labels that maps a label name to the list of values that are allowed
    /// for that label. Only set if detailed labels are requested.
    pub permitted_labels: Option<HashMap<String, Vec<String>>>,
    /// The reviewers that can be removed by the calling user as a list of AccountInfo entities.
    /// Only set if detailed labels are requested.
    pub removable_reviewers: Option<Vec<AccountInfo>>,
    /// The reviewers as a map that maps a reviewer state to a list of AccountInfo entities.
    pub reviewers: Option<HashMap<ReviewerState, Vec<AccountInfo>>>,
    /// Updates to reviewers that have been made while the change was in the WIP state.
    /// Only present on WIP changes and only if there are pending reviewer updates to report.
    /// These are reviewers who have not yet been notified about being added to or removed from the change.
    /// Only set if detailed labels are requested.
    pub pending_reviewers: Option<HashMap<ReviewerState, Vec<AccountInfo>>>,
    /// Updates to reviewers set for the change as ReviewerUpdateInfo entities.
    /// Only set if reviewer updates are requested and if NoteDb is enabled.
    pub reviewer_updates: Option<Vec<ReviewerUpdateInfo>>,
    /// Messages associated with the change as a list of ChangeMessageInfo entities.
    /// Only set if messages are requested.
    pub messages: Option<Vec<ChangeMessageInfo>>,
    /// The commit ID of the current patch set of this change.
    /// Only set if the current revision is requested or if all revisions are requested.
    pub current_revision: Option<String>,
    /// All patch sets of this change as a map that maps the commit ID of the patch set
    /// to a RevisionInfo entity. Only set if the current revision is requested (in which case
    /// it will only contain a key for the current revision) or if all revisions are requested.
    pub revisions: Option<HashMap<String, RevisionInfo>>,
    /// A list of TrackingIdInfo entities describing references to external tracking systems.
    /// Only set if tracking ids are requested.
    pub tracking_ids: Option<Vec<TrackingIdInfo>>,
    /// Whether the query would deliver more results if not limited.
    /// Only set on the last change that is returned.
    #[serde(default, rename = "_more_changes")]
    pub more_changes: bool,
    /// A list of ProblemInfo entities describing potential problems with this change.
    /// Only set if CHECK is set.
    pub problems: Option<Vec<ProblemInfo>>,
    /// When present, change is marked as private.
    #[serde(default)]
    pub is_private: bool,
    /// When present, change is marked as Work In Progress.
    #[serde(default)]
    pub work_in_progress: bool,
    /// When present, change has been marked Ready at some point in time.
    #[serde(default)]
    pub has_review_started: bool,
    /// The numeric Change-Id of the change that this change reverts.
    pub revert_of: Option<u32>,
    /// ID of the submission of this change. Only set if the status is MERGED.
    pub submission_id: Option<String>,
}

/// The ChangeInput entity contains information about creating a new change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeInput {
    /// The name of the project.
    pub project: String,
    /// The name of the target branch.
    /// The refs/heads/ prefix is omitted.
    pub branch: String,
    /// The commit message of the change. Comment lines (beginning with #) will be removed.
    pub subject: String,
    /// The topic to which this change belongs.
    pub topic: Option<String>,
    /// The status of the change (only NEW accepted here).
    pub status: Option<ChangeStatus>,
    /// Whether the new change should be marked as private.
    pub is_private: Option<bool>,
    /// Whether the new change should be set to work in progress.
    pub work_in_progress: Option<bool>,
    /// A {change-id} that identifies the base change for a create change operation.
    /// Mutually exclusive with base_commit.
    pub base_change: Option<String>,
    /// A 40-digit hex SHA-1 of the commit which will be the parent commit of the newly
    /// created change. If set, it must be a merged commit on the destination branch.
    /// Mutually exclusive with base_change.
    pub base_commit: Option<String>,
    /// Allow creating a new branch when set to true.
    /// Using this option is only possible for non-merge commits (if the merge field is not set).
    pub new_branch: Option<bool>,
    /// The detail of a merge commit as a MergeInput entity.
    /// If set, the target branch (see branch field) must exist (it is not possible to create it
    /// automatically by setting the new_branch field to true.
    pub merge: Option<MergeInput>,
    /// An AccountInput entity that will set the author of the commit to create.
    /// The author must be specified as name/email combination.
    /// The caller needs "Forge Author" permission when using this field.
    /// This field does not affect the owner of the change, which will continue to use the identity
    /// of the caller.
    pub author: Option<AccountInput>,
    /// Notify handling that defines to whom email notifications should be sent after the change is
    /// created. If not set, the default is ALL.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
}

/// Change kind.
#[derive(Debug, Display, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeKind {
    Rework,
    TrivialRebase,
    MergeFirstParentUpdate,
    NoCodeChange,
    NoChange,
}

/// The ChangeMessageInfo entity contains information about a message attached to a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ChangeMessageInfo {
    /// The ID of the message.
    pub id: String,
    /// Author of the message as an AccountInfo entity.
    /// Unset if written by the Gerrit system.
    pub author: Option<AccountInfo>,
    /// Real author of the message as an AccountInfo entity.
    /// Set if the message was posted on behalf of another user.
    pub read_author: Option<AccountInfo>,
    /// The timestamp this message was posted.
    pub date: Timestamp,
    /// The text left by the user.
    pub message: String,
    /// Value of the tag field from ReviewInput set while posting the review.
    /// Votes/comments that contain tag with 'autogenerated:' prefix can be filtered out in the web UI.
    /// NOTE: To apply different tags on different votes/comments multiple invocations of the REST call are required.
    pub tag: String,
    /// Which patchset (if any) generated this message.
    #[serde(rename = "_revision_number")]
    pub revision_number: Option<u32>,
}

/// The status of a change.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeStatus {
    New,
    Merged,
    Abandoned,
    Draft,
}

/// The type of change.
#[derive(Debug, Display, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ChangeType {
    Added,
    Modified,
    Deleted,
    Renamed,
    Copied,
    Rewrite,
}

/// The CherryPickInput entity contains information for cherry-picking a change to a new branch.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct CherryPickInput {
    /// Commit message for the cherry-pick change. If not set, the commit message of the
    /// cherry-picked commit is used.
    pub message: Option<String>,
    /// Destination branch.
    pub destination: String,
    /// 40-hex digit SHA-1 of the commit which will be the parent commit of the newly created change.
    /// If set, it must be a merged commit or a change revision on the destination branch.
    pub base: Option<String>,
    /// Number of the parent relative to which the cherry-pick should be considered.
    pub parent: Option<u32>,
    /// Notify handling that defines to whom email notifications should be sent after the change is
    /// created. If not set, the default is NONE.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
    /// If true, carries reviewers and ccs over from original change to newly created one.
    pub keep_reviewers: Option<bool>,
    /// If true, the cherry-pick uses content merge and succeeds also if there are conflicts.
    /// If there are conflicts the file contents of the created change contain git conflict markers
    /// to indicate the conflicts. Callers can find out if there were conflicts by checking the
    /// contains_git_conflicts field in the ChangeInfo. If there are conflicts the cherry-pick
    /// change is marked as work-in-progress.
    pub allow_conflicts: Option<bool>,
}

/// The CommentInfo entity contains information about an inline comment.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct CommentInfo {
    /// The patch set number for the comment; only set in contexts where
    /// comments may be returned for multiple patch sets.
    pub patch_set: Option<u32>,
    /// The URL encoded UUID of the comment.
    pub id: String,
    /// The path of the file for which the inline comment was done.
    /// Not set if returned in a map where the key is the file path.
    pub path: Option<String>,
    /// The side on which the comment was added.
    /// Allowed values are REVISION and PARENT. If not set, the default is REVISION.
    pub side: Option<CommentSide>,
    /// The 1-based parent number. Used only for merge commits when side == PARENT.
    /// When not set the comment is for the auto-merge tree.
    pub parent: Option<String>,
    /// The number of the line for which the comment was done.
    /// If range is set, this equals the end line of the range.
    /// If neither line nor range is set, it’s a file comment.
    pub line: Option<u32>,
    /// The range of the comment as a CommentRange entity.
    pub range: Option<CommentRange>,
    /// The URL encoded UUID of the comment to which this comment is a reply.
    pub in_reply_to: Option<String>,
    /// The comment message.
    pub message: Option<String>,
    /// The timestamp of when this comment was written.
    pub updated: Timestamp,
    /// The author of the message as an AccountInfo entity.
    /// Unset for draft comments, assumed to be the calling user.
    pub author: Option<AccountInfo>,
    /// Value of the tag field from ReviewInput set while posting the review.
    /// NOTE: To apply different tags on different votes/comments multiple invocations of the REST call are required.
    pub tag: Option<String>,
    /// Whether or not the comment must be addressed by the user.
    /// The state of resolution of a comment thread is stored in the last comment in that thread chronologically.
    pub unresolved: Option<bool>,
}

/// The CommentInput entity contains information for creating an inline comment.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct CommentInput {
    /// The URL encoded UUID of the comment if an existing draft comment should be updated.
    pub id: Option<String>,
    /// The path of the file for which the inline comment should be added.
    /// Doesn’t need to be set if contained in a map where the key is the file path.
    pub path: Option<String>,
    /// The side on which the comment was added.
    /// Allowed values are REVISION and PARENT. If not set, the default is REVISION.
    pub side: Option<CommentSide>,
    /// The number of the line for which the comment should be added.
    /// 0 if it is a file comment.
    /// If neither line nor range is set, a file comment is added.
    /// If range is set, this value is ignored in favor of the end_line of the range.
    pub line: Option<u32>,
    /// The range of the comment as a CommentRange entity.
    pub range: Option<CommentRange>,
    /// The URL encoded UUID of the comment to which this comment is a reply.
    pub in_reply_to: Option<String>,
    /// The timestamp of when this comment was written.
    /// Accepted but ignored.
    pub updated: Timestamp,
    /// The comment message.
    /// If not set and an existing draft comment is updated, the existing draft comment is deleted.
    pub message: Option<String>,
    /// Value of the tag field. Only allowed on draft comment inputs;
    /// for published comments, use the tag field in ReviewInput.
    /// Votes/comments that contain tag with 'autogenerated:' prefix can be filtered out in the web UI.
    pub tag: Option<String>,
    /// Whether or not the comment must be addressed by the user.
    /// This value will default to false if the comment is an orphan, or the value of the
    /// in_reply_to comment if it is supplied.
    pub unresolved: Option<bool>,
}

/// The CommentRange entity describes the range of an inline comment.
/// The comment range is a range from the start position, specified by start_line and
/// start_character, to the end position, specified by end_line and end_character.
/// The start position is inclusive and the end position is exclusive.
/// So, a range over part of a line will have start_line equal to end_line;
/// however a range with end_line set to 5 and end_character equal to 0 will not include any
/// characters on line 5,
#[derive(Debug, Serialize, Deserialize)]
pub struct CommentRange {
    /// The start line number of the range. (1-based)
    pub start_line: u32,
    /// The character position in the start line. (0-based)
    pub start_character: u32,
    /// The end line number of the range. (1-based)
    pub end_line: u32,
    /// The character position in the end line. (0-based)
    pub end_character: u32,
}

/// The CommitInfo entity contains information about a commit.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitInfo {
    /// The commit ID. Not set if included in a RevisionInfo entity that is contained in a map
    /// which has the commit ID as key.
    pub commit: Option<String>,
    /// The parent commits of this commit as a list of CommitInfo entities.
    /// In each parent, only the commit and subject fields are populated.
    pub parents: Option<Vec<CommitInfo>>,
    /// The author of the commit as a GitPersonInfo entity.
    pub author: Option<GitPersonInfo>,
    /// The committer of the commit as a GitPersonInfo entity.
    pub committer: Option<GitPersonInfo>,
    /// The subject of the commit (header line of the commit message).
    pub subject: String,
    /// The commit message.
    pub message: Option<String>,
    /// Links to the commit in external sites as a list of WebLinkInfo entities.
    pub web_links: Option<WebLinkInfo>,
}

/// The CommitMessageInput entity contains information for changing the commit message of a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct CommitMessageInput {
    pub message: String,
    /// Notify handling that defines to whom email notifications should be sent after
    /// the commit message was updated.
    /// If not set, the default is OWNER for WIP changes and ALL otherwise.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
}

/// The side on which the comment was added.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum CommentSide {
    Revision,
    Parent,
}

/// The DeleteChangeMessageInput entity contains the options for deleting a change message.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteChangeMessageInput {
    /// The reason why the change message should be deleted.
    /// If set, the change message will be replaced with:
    /// "Change message removed by: name\nReason: reason`", or just "Change message removed by: `name." if not set.
    pub reason: Option<String>,
}

/// The DeleteCommentInput entity contains the option for deleting a comment.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteCommentInput {
    /// The reason why the comment should be deleted.
    /// If set, the comment’s message will be replaced with:
    /// "Comment removed by: name; Reason: reason`", or just "Comment removed by: `name." if not set.
    pub reason: Option<String>,
}

/// The DeleteReviewerInput entity contains options for the deletion of a reviewer.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteReviewerInput {
    /// Notify handling that defines to whom email notifications should be sent
    /// after the reviewer is deleted.
    /// If not set, the default is ALL.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
}

/// The DeleteVoteInput entity contains options for the deletion of a vote.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteVoteInput {
    /// The label for which the vote should be deleted.
    /// If set, must match the label in the URL.
    pub label: Option<String>,
    /// Notify handling that defines to whom email notifications should be sent after
    /// the vote is deleted.
    /// If not set, the default is ALL.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
}

/// The DescriptionInput entity contains information for setting a description.
#[derive(Debug, Serialize, Deserialize)]
pub struct DescriptionInput {
    /// The description text.
    pub description: String,
}

/// The DiffContent entity contains information about the content differences in a file.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct DiffContent {
    /// Content only in the file on side A (deleted in B).
    pub a: Option<String>,
    /// Content only in the file on side B (added in B).
    pub b: Option<String>,
    /// Content in the file on both sides (unchanged).
    pub ab: Option<String>,
    /// Text sections deleted from side A as a DiffIntralineInfo entity.
    /// Only present when the intraline parameter is set and the DiffContent is a replace,
    /// i.e. both a and b are present
    pub edit_a: Option<String>,
    /// Text sections inserted in side B as a DiffIntralineInfo entity.
    /// Only present when the intraline parameter is set and the DiffContent is a replace,
    /// i.e. both a and b are present
    pub edit_b: Option<String>,
    /// Indicates whether this entry was introduced by a rebase.
    #[serde(default)]
    pub due_to_rebase: bool,
    /// Count of lines skipped on both sides when the file is too large to include all common lines.
    pub skip: Option<i32>,
    /// Set to true if the region is common according to the requested ignore-whitespace parameter,
    /// but a and b contain differing amounts of whitespace. When present and true a and b are used instead of ab.
    pub common: Option<bool>,
}

/// The DiffFileMetaInfo entity contains meta information about a file diff.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct DiffFileMetaInfo {
    /// The name of the file.
    pub name: String,
    /// The content type of the file.
    pub content_type: String,
    /// The total number of lines in the file.
    pub lines: u32,
    /// Links to the file in external sites as a list of WebLinkInfo entries.
    pub web_links: Option<Vec<WebLinkInfo>>,
}

/// The DiffInfo entity contains information about the diff of a file in a revision.
/// If the weblinks-only parameter is specified, only the web_links field is set.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct DiffInfo {
    /// Meta information about the file on side A as a DiffFileMetaInfo entity.
    /// Not present when the file is added.
    pub meta_a: Option<DiffFileMetaInfo>,
    /// Meta information about the file on side B as a DiffFileMetaInfo entity.
    /// Not present when the file is deleted.
    pub meta_b: Option<DiffFileMetaInfo>,
    /// The type of change
    pub change_type: ChangeType,
    /// The Intraline status.
    /// Only set when the intraline parameter was specified in the request.
    pub intraline_status: Option<IntralineStatus>,
    /// A list of strings representing the patch set diff header.
    pub diff_header: Vec<String>,
    /// The content differences in the file as a list of DiffContent entities.
    pub content: Vec<DiffContent>,
    /// Links to the file diff in external sites as a list of DiffWebLinkInfo entries.
    pub web_links: Option<DiffWebLinkInfo>,
    /// Whether the file is binary.
    #[serde(default)]
    pub binary: bool,
}

/// The DiffIntralineInfo entity contains information about intraline edits in a file.
///
/// The information consists of a list of <skip length, edit length> pairs, where the skip length is
/// the number of characters between the end of the previous edit and the start of this edit, and
/// the edit length is the number of edited characters following the skip.
/// The start of the edits is from the beginning of the related diff content lines.
/// If the list is empty, the entire DiffContent should be considered as unedited.
///
/// Note that the implied newline character at the end of each line is included in the
/// length calculation,and thus it is possible for the edits to span newlines.
#[derive(Debug, Serialize, Deserialize)]
pub struct DiffIntralineInfo {
    #[serde(flatten)]
    pub values: Vec<String>,
}

/// The DiffWebLinkInfo entity describes a link on a diff screen to an external site.
#[derive(Debug, Serialize, Deserialize)]
pub struct DiffWebLinkInfo {
    /// The link name.
    pub name: String,
    /// The link URL.
    pub url: String,
    /// URL to the icon of the link.
    pub image_url: String,
    /// Whether the web link should be shown on the side-by-side diff screen.
    pub show_on_side_by_side_diff_view: bool,
    /// Whether the web link should be shown on the unified diff screen.
    pub show_on_unified_diff_view: bool,
}

/// Draft handling.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum DraftHandling {
    Publish,
    PublishAllRevisions,
    Keep,
}

/// The EditFileInfo entity contains additional information of a file within a change edit.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct EditFileInfo {
    /// Links to the diff info in external sites as a list of WebLinkInfo entities.
    pub wbe_links: Option<Vec<WebLinkInfo>>,
}

/// The EditInfo entity contains information about a change edit.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct EditInfo {
    /// The commit of change edit as CommitInfo entity.
    pub commit: CommitInfo,
    /// The patch set number of the patch set the change edit is based on.
    pub base_patch_set_number: u32,
    /// The revision of the patch set the change edit is based on.
    pub base_revision: u32,
    /// The ref of the change edit.
    #[serde(rename = "ref")]
    pub refspec: String,
    /// Information about how to fetch this patch set.
    /// The fetch information is provided as a map that maps the protocol name (“git”, “http”, “ssh”)
    /// to FetchInfo entities.
    pub fetch: Option<BTreeMap<String, FetchInfo>>,
    /// The files of the change edit as a map that maps the file names to FileInfo entities.
    pub files: Option<BTreeMap<String, FileInfo>>,
}

/// The FetchInfo entity contains information about how to fetch a patch set via a certain protocol.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct FetchInfo {
    pub url: String,
    /// The ref of the patch set.
    #[serde(rename = "ref")]
    pub refspec: String,
    /// The download commands for this patch set as a map that maps the command names to the commands.
    /// Only set if download commands are requested.
    pub commands: Option<HashMap<String, String>>,
}

/// The FileInfo entity contains information about a file in a patch set.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct FileInfo {
    /// The status of the file
    #[serde(default)]
    pub status: FileStatus,
    /// Whether the file is binary.
    #[serde(default)]
    pub binary: bool,
    /// The old file path.
    /// Only set if the file was renamed or copied.
    pub old_path: Option<String>,
    /// Number of inserted lines.
    /// Not set for binary files or if no lines were inserted.
    /// An empty last line is not included in the count and hence this number can differ by one
    /// from details provided in <<#diff-info,DiffInfo>>.
    pub lines_inserted: Option<u32>,
    /// Number of deleted lines.
    /// Not set for binary files or if no lines were deleted.
    /// An empty last line is not included in the count and hence this number can differ by one
    /// from details provided in <<#diff-info,DiffInfo>>.
    pub lines_deleted: Option<u32>,
    /// Number of bytes by which the file size increased/decreased.
    pub size_delta: i32,
    /// File size in bytes.
    pub size: Option<u32>,
}

/// File status.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum FileStatus {
    #[serde(rename = "M")]
    Modified,
    #[serde(rename = "A")]
    Added,
    #[serde(rename = "D")]
    Deleted,
    #[serde(rename = "R")]
    Renamed,
    #[serde(rename = "C")]
    Copied,
    #[serde(rename = "W")]
    Rewritten,
}

impl Default for FileStatus {
    fn default() -> Self {
        FileStatus::Modified
    }
}

/// The FixInput entity contains options for fixing commits using the fix change endpoint.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct FixInput {
    /// If true, delete patch sets from the database if they refer to missing commit options.
    pub delete_patch_set_if_commit_missing: bool,
    /// If set, check that the change is merged into the destination branch as this exact SHA-1.
    /// If not, insert a new patch set referring to this commit.
    pub expect_merged_as: Option<String>,
}

/// The FixSuggestionInfo entity represents a suggested fix.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct FixSuggestionInfo {
    /// The UUID of the suggested fix.
    /// It will be generated automatically and hence will be ignored if it’s set for input objects.
    pub fix_id: Option<String>,
    /// A description of the suggested fix.
    pub description: String,
    /// A list of FixReplacementInfo entities indicating how the content of one or several files
    /// should be modified. Within a file, they should refer to non-overlapping regions.
    pub replacements: Vec<FixReplacementInfo>,
}

/// The FixReplacementInfo entity describes how the content of a file should be replaced by another content.
#[derive(Debug, Serialize, Deserialize)]
pub struct FixReplacementInfo {
    /// The path of the file which should be modified. Any file in the repository may be modified.
    pub path: String,
    /// A CommentRange indicating which content of the file should be replaced.
    /// Lines in the file are assumed to be separated by the line feed character,
    /// the carriage return character, the carriage return followed by the line feed character,
    /// or one of the other Unicode linebreak sequences supported by Java.
    pub range: CommentRange,
    /// The content which should be used instead of the current one.
    pub replacement: String,
}

/// The GitPersonInfo entity contains information about the author/committer of a commit.
#[derive(Debug, Serialize, Deserialize)]
pub struct GitPersonInfo {
    /// The name of the author/committer.
    pub name: String,
    /// The email address of the author/committer.
    pub email: String,
    /// The timestamp of when this identity was constructed.
    pub date: Timestamp,
    /// The timezone offset from UTC of when this identity was constructed.
    pub tz: i32,
}

/// The GroupBaseInfo entity contains base information about the group.
#[derive(Debug, Serialize, Deserialize)]
pub struct GroupBaseInfo {
    /// The UUID of the group.
    pub id: String,
    /// The name of the group.
    pub name: String,
}

/// The HashtagsInput entity contains information about hashtags to add to, and/or remove from, a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct HashtagsInput {
    /// The list of hashtags to be added to the change.
    pub add: Option<Vec<String>>,
    /// The list of hashtags to be removed from the change.
    pub remove: Option<Vec<String>>,
}

/// Common HTTP methods to cause state changes.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum HttpMethod {
    Post,
    Put,
    Delete,
}

/// The IncludedInInfo entity contains information about the branches a change was merged into and tags it was tagged with.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct IncludedInInfo {
    /// The list of branches this change was merged into. Each branch is listed without the 'refs/head/' prefix.
    pub branches: Vec<String>,
    /// The list of tags this change was tagged with. Each tag is listed without the 'refs/tags/' prefix.
    pub tags: Vec<String>,
    /// A map that maps a name to a list of external systems that include this change,
    /// e.g. a list of servers on which this change is deployed.
    pub external: Option<HashMap<String, String>>,
}

/// The Intraline status.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum IntralineStatus {
    Ok,
    Error,
    Timeout,
}

/// The LabelInfo entity contains information about a label on a change, always corresponding to the
/// current patch set.
/// There are two options that control the contents of LabelInfo: LABELS and DETAILED_LABELS.
///  - For a quick summary of the state of labels, use LABELS.
///  - For detailed information about labels, including exact numeric votes for all users and the
///    allowed range of votes for the current user, use DETAILED_LABELS.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct LabelInfo {
    /// Whether the label is optional. Optional means the label may be set,
    /// but it’s neither necessary for submission nor does it block submission if set.
    #[serde(default)]
    pub optional: bool,
    /// One user who approved this label on the change (voted the maximum value) as an AccountInfo.
    pub approved: Option<AccountInfo>,
    /// One user who rejected this label on the change (voted the minimum value) as an AccountInfo.
    pub rejected: Option<AccountInfo>,
    /// One user who recommended this label on the change (voted positively,
    /// but not the maximum value) as an AccountInfo entity.
    pub recommended: Option<AccountInfo>,
    /// One user who disliked this label on the change (voted negatively, but not the minimum value)
    /// as an AccountInfo entity.
    pub disliked: Option<AccountInfo>,
    /// If true, the label blocks submit operation. If not set, the default is false.
    #[serde(default)]
    pub blocking: bool,
    /// List of all approvals for this label as a list of ApprovalInfo entities. Items in this list
    /// may not represent actual votes cast by users; if a user votes on any label, a corresponding
    /// ApprovalInfo will appear in this list for all labels.
    pub all: Option<Vec<ApprovalInfo>>,
    /// The voting value of the user who recommended/disliked this label on the change
    /// if it is not “+1”/“-1”.
    pub value: Option<i32>,
    /// The default voting value for the label. This value may be outside the range specified
    /// in permitted_labels.
    pub default_value: Option<i32>,
    /// A map of all values that are allowed for this label.
    /// The map maps the values (“-2”, “-1”, " `0`", “+1”, “+2”) to the value descriptions.
    pub values: Option<HashMap<String, String>>,
}

/// The MergeableInfo entity contains information about the mergeability of a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct MergeableInfo {
    /// Submit type used for this change.
    pub submit_type: SubmitType,
    /// The strategy of the merge.
    pub strategy: Option<MergeStrategy>,
    /// true if this change is cleanly mergeable, false otherwise
    pub mergeable: bool,
    /// true if this change is already merged, false otherwise
    pub commit_merged: Option<bool>,
    /// true if the content of this change is already merged, false otherwise
    pub content_merged: Option<bool>,
    /// A list of paths with conflicts.
    pub conflicts: Option<Vec<String>>,
    /// A list of other branch names where this change could merge cleanly.
    pub mergeable_into: Option<Vec<String>>,
}

/// The MergeInput entity contains information about the merge.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct MergeInput {
    /// The source to merge from, e.g. a complete or abbreviated commit SHA-1,a complete reference
    /// name, a short reference name under refs/heads, refs/tags, or refs/remotes namespace, etc.
    pub source: String,
    /// A branch from which source is reachable. If specified, source is checked for visibility and
    /// reachability against only this branch. This speeds up the operation, especially for large
    /// repos with many branches.
    pub source_branch: Option<String>,
    /// The strategy of the merge.
    pub strategy: Option<MergeStrategy>,
    /// If true, creating the merge succeeds also if there are conflicts.
    /// If there are conflicts the file contents of the created change contain git conflict markers to indicate the conflicts.
    /// Callers can find out whether there were conflicts by checking the contains_git_conflicts field in the ChangeInfo.
    /// If there are conflicts the change is marked as work-in-progress.
    /// This option is not supported for all merge strategies (e.g. it’s supported for recursive and resolve, but not for simple-two-way-in-core).
    /// Defaults to false.
    pub allow_conflicts: Option<bool>,
}

/// The MergePatchSetInput entity contains information about updating a new change by creating
/// a new merge commit.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct MergePatchSetInput {
    /// The new subject for the change, if not specified, will reuse the current patch set’s subject
    pub subject: Option<String>,
    /// Use the current patch set’s first parent as the merge tip when set to true.
    /// Default to false.
    pub inherit_parent: Option<bool>,
    /// A {change-id} that identifies a change. When inherit_parent is false, the merge tip will be
    /// the current patch set of the base_change if it’s set. Otherwise, the current branch tip of
    /// the destination branch will be used.
    pub base_change: Option<String>,
    /// The detail of the source commit for merge as a MergeInput entity.
    pub merge: MergeInput,
}

/// The strategy of the merge.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum MergeStrategy {
    Recursive,
    Resolve,
    SimpleTwoWayInCore,
    Ours,
    Theirs,
}

/// The MoveInput entity contains information for moving a change to a new branch.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct MoveInput {
    /// Destination branch.
    pub destination_branch: String,
    /// A message to be posted in this change’s comments
    pub message: Option<String>,
}

/// Notify handling that defines to whom email notifications should be sent.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum NotifyHandling {
    All,
    None,
    Owner,
    OwnerReviewers,
}

/// The NotifyInfo entity contains detailed information about who should be notified about an
/// update. These notifications are sent out even if a notify option in the request input disables
/// normal notifications. NotifyInfo entities are normally contained in a notify_details map in the
/// request input where the key is the recipient type.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct NotifyInfo {
    /// A list of account IDs that identify the accounts that should be should be notified.
    pub accounts: Option<Vec<String>>,
}

/// The PrivateInput entity contains information for changing the private flag on a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct PrivateInput {
    /// Message describing why the private flag was changed.
    pub message: Option<String>,
}

/// The ProblemInfo entity contains a description of a potential consistency problem with a change.
/// These are not related to the code review process, but rather indicate some inconsistency in
/// Gerrit’s database or repository metadata related to the enclosing change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ProblemInfo {
    /// Plaintext message describing the problem with the change.
    pub message: String,
    /// The status of the problem.
    pub status: Option<ProblemStatus>,
    /// If status is set, an additional plaintext message describing the outcome of the fix.
    pub outcome: Option<String>,
}

/// The status of the problem.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ProblemStatus {
    Fixed,
    FixFailed,
}

/// The PublishChangeEditInput entity contains options for the publishing of change edit.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct PublishChangeEditInput {
    /// Notify handling that defines to whom email notifications should be sent
    /// after the change edit is published.
    /// If not set, the default is ALL.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
}

/// The PureRevertInfo entity describes the result of a pure revert check.
#[derive(Debug, Serialize, Deserialize)]
pub struct PureRevertInfo {
    /// Outcome of the check as boolean.
    pub is_pure_revert: bool,
}

/// The PushCertificateInfo entity contains information about a push certificate provided when
/// the user pushed for review with git push --signed HEAD:refs/for/<branch>.
/// Only used when signed push is enabled on the server.
#[derive(Debug, Serialize, Deserialize)]
pub struct PushCertificateInfo {
    /// Signed certificate payload and GPG signature block.
    pub certificate: String,
    /// Information about the key that signed the push, along with any problems found while checking
    /// the signature or the key itself, as a GpgKeyInfo entity.
    pub key: GpgKeyInfo,
}

/// The RangeInfo entity stores the coordinates of a range.
#[derive(Debug, Serialize, Deserialize)]
pub struct RangeInfo {
    /// First index.
    pub start: u32,
    /// Last index.
    pub end: u32,
}

/// The RebaseInput entity contains information for changing parent when rebasing.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct RebaseInput {
    /// The new parent revision. This can be a ref or a SHA1 to a concrete patchset.
    /// Alternatively, a change number can be specified, in which case the current patch set is inferred.
    /// Empty string is used for rebasing directly on top of the target branch, which effectively breaks
    /// dependency towards a parent change.
    pub base: Option<String>,
}

/// The recipient type for notification handling.
#[derive(Debug, Display, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum RecipientType {
    To,
    Cc,
    Bcc,
}

/// The RelatedChangeAndCommitInfo entity contains information about a related change and commit.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedChangeAndCommitInfo {
    /// The project of the change or commit.
    pub project: String,
    /// The Change-Id of the change.
    pub change_id: Option<String>,
    /// The commit as a CommitInfo entity.
    pub commit: CommitInfo,
    /// The change number.
    #[serde(rename = "_change_number")]
    pub change_number: Option<u32>,
    /// The revision number.
    #[serde(rename = "_revision_number")]
    pub revision_number: Option<u32>,
    /// The current revision number.
    #[serde(rename = "_current_revision_number")]
    pub current_revision_number: Option<u32>,
    /// The status of the change.
    pub status: Option<ChangeStatus>,
}

/// The RelatedChangesInfo entity contains information about related changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct RelatedChangesInfo {
    /// A list of RelatedChangeAndCommitInfo entities describing the related changes.
    /// Sorted by git commit order, newest to oldest. Empty if there are no related changes.
    pub changes: Vec<RelatedChangeAndCommitInfo>,
}

/// The Requirement entity contains information about a requirement relative to a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct Requirement {
    /// Status of the requirement.
    pub status: RequirementStatus,
    /// A human readable reason.
    #[serde(rename = "fallbackText")]
    pub fallback_text: String,
    /// Alphanumerical (plus hyphens or underscores) string to identify what the requirement is and
    /// why it was triggered. Can be seen as a class: requirements sharing the same type were
    /// created for a similar reason, and the data structure will follow one set of rules.
    #[serde(rename = "type")]
    pub req_type: String,
    /// Holds custom key-value strings, used in templates to render richer status messages.
    pub data: Option<String>,
}

/// Status of the requirement.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum RequirementStatus {
    Ok,
    NotReady,
    RuleError,
}

/// The RestoreInput entity contains information for restoring a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct RestoreInput {
    /// Message to be added as review comment to the change when restoring the change.
    pub message: Option<String>,
}

/// The RevertInput entity contains information for reverting a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct RevertInput {
    /// Message to be added as review comment to the change when reverting the change.
    pub message: Option<String>,
    /// Notify handling that defines to whom email notifications should be sent
    /// for reverting the change.
    /// If not set, the default is ALL.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
    /// Name of the topic for the revert change.
    /// If not set, the default for Revert endpoint is the topic of the change being reverted,
    /// and the default for the RevertSubmission endpoint is revert-{submission_id}-{timestamp.now}.
    pub topic: Option<String>,
}

/// The RevertSubmissionInfo entity describes the revert changes.
#[derive(Debug, Serialize, Deserialize)]
pub struct RevertSubmissionInfo {
    /// A list of ChangeInfo that describes the revert changes.
    /// Each entity in that list is a revert change that was created in that revert submission.
    pub revert_changes: Vec<ChangeInfo>,
}

/// The ReviewInfo entity contains information about a review.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewInfo {
    /// The labels of the review as a map that maps the label names to the voting values.
    pub labels: BTreeMap<String, i32>,
}

/// The Reviewer State
#[derive(Debug, Display, PartialEq, Eq, Clone, Hash, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum ReviewerState {
    /// Users with at least one non-zero vote on the change.
    Reviewer,
    /// Users that were added to the change, but have not voted.
    Cc,
    /// Users that were previously reviewers on the change, but have been removed.
    Removed,
}

/// The ReviewerUpdateInfo entity contains information about updates to change’s reviewers set.
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewerUpdateInfo {
    /// Timestamp of the update.
    pub updated: Timestamp,
    /// The account which modified state of the reviewer in question as AccountInfo entity.
    pub updated_by: AccountInfo,
    /// The reviewer account added or removed from the change as an AccountInfo entity.
    pub reviewer: AccountInfo,
    /// The reviewer state.
    pub state: ReviewerState,
}

/// The ReviewInput entity contains information for adding a review to a revision.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewInput {
    /// The message to be added as review comment.
    pub message: Option<String>,
    /// Apply this tag to the review comment message, votes, and inline comments.
    /// Tags may be used by CI or other automated systems to distinguish them from human reviews.
    /// Votes/comments that contain tag with 'autogenerated:' prefix can be filtered out in the web UI.
    pub tag: Option<String>,
    /// The votes that should be added to the revision as a map that maps the label names to the voting values.
    pub labels: Option<BTreeMap<String, i32>>,
    /// The comments that should be added as a map that maps a file path to a list of CommentInput entities.
    pub comments: Option<HashMap<String, Vec<CommentInput>>>,
    /// The robot comments that should be added as a map that maps a file path to a list of RobotCommentInput entities.
    pub robot_comments: Option<HashMap<String, Vec<RobotCommentInput>>>,
    /// Draft handling that defines how draft comments are handled that are already in the database
    /// but that were not also described in this input.
    /// Allowed values are PUBLISH, PUBLISH_ALL_REVISIONS and KEEP.
    /// All values except PUBLISH_ALL_REVISIONS operate only on drafts for a single revision.
    /// Only KEEP is allowed when used in conjunction with on_behalf_of.
    /// If not set, the default is KEEP. If on_behalf_of is set, then no other value besides KEEP is allowed.
    pub drafts: Option<DraftHandling>,
    /// Notify handling that defines to whom email notifications should be sent after the review is stored.
    /// If not set, the default is ALL.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
    /// If true, comments with the same content at the same place will be omitted.
    pub omit_duplicate_comments: Option<bool>,
    /// {account-id} the review should be posted on behalf of.
    /// To use this option the caller must have been granted labelAs-NAME permission for all keys of labels.
    pub on_behalf_of: Option<String>,
    /// A list of ReviewerInput representing reviewers that should be added to the change.
    pub reviewers: Option<Vec<ReviewerInput>>,
    /// If true, and if the change is work in progress, then start review.
    /// It is an error for both ready and work_in_progress to be true.
    pub ready: Option<bool>,
    /// If true, mark the change as work in progress.
    /// It is an error for both ready and work_in_progress to be true.
    pub work_in_progress: Option<bool>,
}

/// The ReviewerInfo entity contains information about a reviewer and its votes on a change.
/// ReviewerInfo has the same fields as AccountInfo and includes detailed account information.
/// In addition ReviewerInfo has the following fields:
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewerInfo {
    /// The account information entity.
    #[serde(flatten)]
    pub account: AccountInfo,
    /// The approvals of the reviewer as a map that maps the label names to
    /// the approval values (“-2”, “-1”, “0”, “+1”, “+2”).
    pub approvals: BTreeMap<String, i32>,
}

/// The ReviewerInput entity contains information for adding a reviewer to a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct ReviewerInput {
    /// The ID of one account that should be added as reviewer or the ID of one internal group for
    /// which all members should be added as reviewers.
    /// If an ID identifies both an account and a group, only the account is added as reviewer to the change.
    /// External groups, such as LDAP groups, will be silently omitted from a set-review or add-reviewer call.
    pub reviewer: String,
    /// Add reviewer in this state.
    /// If not given, defaults to REVIEWER.
    pub state: Option<ReviewerState>,
    /// Whether adding the reviewer is confirmed.
    /// The Gerrit server may be configured to require a confirmation when adding a group as
    /// reviewer that has many members.
    pub confirmed: Option<bool>,
    /// Notify handling that defines to whom email notifications should be sent after
    /// the reviewer is added.
    /// If not set, the default is ALL.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
}

/// The ReviewerInput entity contains information for adding a reviewer to a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct RevisionInfo {
    /// The change kind.
    pub kind: Option<ChangeKind>,
    /// The patch set number, or edit if the patch set is an edit.
    pub _number: u32,
    /// The timestamp of when the patch set was created.
    pub created: Timestamp,
    /// The uploader of the patch set as an AccountInfo entity.
    pub uploader: AccountInfo,
    /// The Git reference for the patch set.
    #[serde(rename = "ref")]
    pub refspec: String,
    /// Information about how to fetch this patch set.
    /// The fetch information is provided as a map that maps the protocol name (“git”, “http”, “ssh”)
    /// to FetchInfo entities. This information is only included if a plugin implementing the
    /// download commands interface is installed.
    pub fetch: HashMap<String, FetchInfo>,
    /// The commit of the patch set as CommitInfo entity.
    pub commit: Option<CommitInfo>,
    /// The files of the patch set as a map that maps the file names to FileInfo entities.
    /// Only set if CURRENT_FILES or ALL_FILES option is requested.
    pub files: Option<BTreeMap<String, FileInfo>>,
    /// Actions the caller might be able to perform on this revision.
    /// The information is a map of view name to ActionInfo entities.
    pub actions: Option<BTreeMap<String, ActionInfo>>,
    /// Indicates whether the caller is authenticated and has commented on the current revision.
    /// Only set if REVIEWED option is requested.
    pub reviewed: Option<bool>,
    /// If the COMMIT_FOOTERS option is requested and this is the current patch set,
    /// contains the full commit message with Gerrit-specific commit footers, as if this revision
    /// were submitted using the Cherry Pick submit type.
    pub commit_with_footers: Option<String>,
    /// If the PUSH_CERTIFICATES option is requested, contains the push certificate provided by the
    /// user when uploading this patch set as a PushCertificateInfo entity.
    /// This field is always set if the option is requested;
    /// if no push certificate was provided, it is set to an empty object.
    pub push_certificate: Option<PushCertificateInfo>,
    /// The description of this patchset, as displayed in the patchset selector menu.
    /// May be null if no description is set.
    pub description: Option<String>,
}

/// The RobotCommentInfo entity contains information about a robot inline comment.
/// RobotCommentInfo has the same fields as CommentInfo. In addition RobotCommentInfo has the following fields:
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct RobotCommentInfo {
    /// The comment information entity.
    #[serde(flatten)]
    pub comment: CommentInfo,
    /// The ID of the robot that generated this comment.
    pub robot_id: String,
    /// An ID of the run of the robot.
    pub robot_run_id: String,
    /// URL to more information.
    pub url: Option<String>,
    /// Robot specific properties as map that maps arbitrary keys to values.
    pub properties: Option<HashMap<String, String>>,
    /// Suggested fixes for this robot comment as a list of FixSuggestionInfo entities.
    pub fix_suggestions: Vec<FixSuggestionInfo>,
}

/// The RobotCommentInput entity contains information for creating an inline robot comment.
/// RobotCommentInput has the same fields as RobotCommentInfo.
#[derive(Debug, Serialize, Deserialize)]
pub struct RobotCommentInput {
    /// The robot comment information entity.
    #[serde(flatten)]
    pub inner: RobotCommentInfo,
}

/// The RuleInput entity contains information to test a Prolog rule.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct RuleInput {
    /// Prolog code to execute instead of the code in refs/meta/config.
    pub rule: String,
    /// When RUN filter rules in the parent projects are called to post-process the results of the
    /// project specific rule. This behavior matches how the rule will execute if installed.
    /// If SKIP the parent filters are not called, allowing the test to return results from the input rule.
    /// RUN if not set.
    pub filters: Option<RuleFilter>,
}

/// Rule filter.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum RuleFilter {
    Run,
    Skip,
}

/// The SubmitInfo entity contains information about the change status after submitting.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitInfo {
    /// The status of the change after submitting is MERGED.
    pub status: ChangeStatus,
    /// The {account-id} of the user on whose behalf the action should be done.
    /// To use this option the caller must have been granted both Submit and Submit (On Behalf Of) permissions.
    /// The user named by on_behalf_of does not need to be granted the Submit permission.
    /// This feature is aimed for CI solutions: the CI account can be granted both permissions,
    /// so individual users don’t need Submit permission themselves.
    /// Still the changes can be submitted on behalf of real users and not with the identity of the CI account.
    pub on_behalf_of: Option<String>,
}

/// The SubmitInput entity contains information for submitting a change.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitInput {
    /// If set, submit the change on behalf of the given user.
    /// The value may take any format accepted by the accounts REST API.
    /// Using this option requires Submit (On Behalf Of) permission on the branch.
    pub on_behalf_of: Option<String>,
    /// Notify handling that defines to whom email notifications should be sent after
    /// the change is submitted.
    /// If not set, the default is ALL.
    pub notify: Option<NotifyHandling>,
    /// Additional information about whom to notify about the update as a
    /// map of recipient type to NotifyInfo entity.
    pub notify_details: Option<HashMap<RecipientType, NotifyInfo>>,
}

/// The SubmitRecord entity describes results from a submit_rule.
/// Fields in this entity roughly correspond to the fields set by LABELS in LabelInfo.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmitRecord {
    /// The submit status.
    pub status: SubmitStatus,
    /// Map of labels that are approved; an AccountInfo identifies the voter chosen by the rule.
    pub ok: Option<BTreeMap<String, AccountInfo>>,
    /// Map of labels that are preventing submit; AccountInfo identifies voter.
    pub reject: Option<BTreeMap<String, AccountInfo>>,
    /// Map of labels that can be used, but do not affect submit.
    /// AccountInfo identifies voter, if the label has been applied.
    pub need: Option<BTreeMap<String, AccountInfo>>,
    /// Map of labels that should have been in need but cannot be used by any user because of access restrictions.
    /// The value is currently an empty object.
    pub impossible: Option<BTreeMap<String, ()>>,
    /// When status is RULE_ERROR this message provides some text describing the failure of the rule predicate.
    pub error_message: Option<String>,
}

/// Submit type.
#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum SubmitType {
    Inherit,
    FastForwardOnly,
    MergeIfNecessary,
    MergeAlways,
    CherryPick,
    RebaseIfNecessary,
    RebaseAlways,
}

impl std::fmt::Display for SubmitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str(match self {
            SubmitType::Inherit => "Inherit",
            SubmitType::FastForwardOnly => "Fast-Forward only",
            SubmitType::MergeIfNecessary => "Merge if Necessary",
            SubmitType::MergeAlways => "Merge Always ",
            SubmitType::CherryPick => "Cherry-Pick",
            SubmitType::RebaseIfNecessary => "Rebase if Necessary",
            SubmitType::RebaseAlways => "Rebase Always",
        })
    }
}

/// The SubmittedTogetherInfo entity contains information about a collection of changes that would be submitted together.
#[derive(Debug, Serialize, Deserialize)]
pub struct SubmittedTogetherInfo {
    /// A list of ChangeInfo entities representing the changes to be submitted together.
    pub changes: Vec<ChangeInfo>,
    /// The number of changes to be submitted together that the current user cannot see.
    /// (This count includes changes that are visible to the current user when their reason for
    /// being submitted together involves changes the user cannot see.)
    pub non_visible_changes: u32,
}

/// Submit status.
#[derive(Debug, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum SubmitStatus {
    /// The change can be submitted.
    Ok,
    /// Additional labels are required before submit.
    NotReady,
    /// Closed changes cannot be submitted.
    Closed,
    /// Rule code failed with an error.
    RuleError,
}

/// The SuggestedReviewerInfo entity contains information about a reviewer that can be added to a
/// change (an account or a group).
/// SuggestedReviewerInfo has either the account field that contains the AccountInfo entity,
/// or the group field that contains the GroupBaseInfo entity.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct SuggestedReviewerInfo {
    /// An AccountInfo entity, if the suggestion is an account.
    pub account: Option<AccountInfo>,
    /// A GroupBaseInfo entity, if the suggestion is a group.
    pub group: Option<GroupBaseInfo>,
    /// The total number of accounts in the suggestion.
    /// This is 1 if account is present. If group is present, the total number of accounts that are
    /// members of the group is returned (this count includes members of nested groups).
    pub count: u32,
    /// True if group is present and count is above the threshold where the confirmed flag must be
    /// passed to add the group as a reviewer.
    pub confirm: Option<bool>,
}

/// The TopicInput entity contains information for setting a topic.
#[derive(Debug, Serialize, Deserialize)]
pub struct TopicInput {
    /// The topic.
    pub topic: String,
}

/// The TrackingIdInfo entity describes a reference to an external tracking system.
#[derive(Debug, Serialize, Deserialize)]
pub struct TrackingIdInfo {
    /// The name of the external tracking system.
    pub system: String,
    /// The tracking id.
    pub id: String,
}

/// The VotingRangeInfo entity describes the continuous voting range from min to max values.
#[derive(Debug, Serialize, Deserialize)]
pub struct VotingRangeInfo {
    /// The minimum voting value.
    pub min: i32,
    /// The maximum voting value.
    pub max: i32,
}

/// The WebLinkInfo entity describes a link to an external site.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct WebLinkInfo {
    /// The link name.
    pub name: String,
    /// The link URL.
    pub url: String,
    /// URL to the icon of the link.
    pub image_url: Option<String>,
}

/// The WorkInProgressInput entity contains additional information for a change set to WorkInProgress/ReadyForReview.
#[skip_serializing_none]
#[derive(Debug, Serialize, Deserialize)]
pub struct WorkInProgressInput {
    /// Message to be added as a review comment to the change being set WorkInProgress/ReadyForReview.
    pub message: Option<String>,
}

// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////
// OPTIONS
// ////////////////////////////////////////////////////////////////////////////////////////////////////////////////////

/// Query parameters available for the change endpoint.
#[derive(Debug, Default, Serialize)]
pub struct QueryParams {
    /// Queries strings for searching changes.
    #[serde(rename = "q", skip_serializing_if = "Option::is_none")]
    pub search_queries: Option<Vec<QueryStr>>,
    /// Additional Options to extend the query results
    #[serde(rename = "o", skip_serializing_if = "Option::is_none")]
    pub additional_opts: Option<Vec<AdditionalOpt>>,
    /// Limit the returned results to no more than X records.
    #[serde(rename = "n", skip_serializing_if = "Option::is_none")]
    pub limit: Option<u32>,
    /// The start query parameter can be supplied to skip a number of changes from the list.
    #[serde(rename = "S", skip_serializing_if = "Option::is_none")]
    pub start: Option<u32>,
}

/// Additional fields can be obtained by adding `o` parameters, each option requires more database
/// lookups and slows down the query response time to the client so they are generally disabled by default.
#[derive(AsRefStr, Display, PartialEq, Eq, Clone, Debug, Serialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum AdditionalOpt {
    /// A summary of each label required for submit, and approvers that have granted (or rejected)
    /// with that label.
    Labels,
    /// Detailed label information, including numeric values of all existing approvals,
    /// recognized label values, values permitted to be set by the current user, all reviewers by state,
    /// and reviewers that may be removed by the current user.
    DetailedLabels,
    /// Describe the current revision (patch set) of the change, including the commit SHA-1 and URLs to fetch from.
    CurrentRevision,
    /// Describe all revisions, not just current.
    AllRevisions,
    /// Include the commands field in the FetchInfo for revisions.
    /// Only valid when the CURRENT_REVISION or ALL_REVISIONS option is selected.
    DownloadCommands,
    /// Parse and output all header fields from the commit object, including message.
    /// Only valid when the CURRENT_REVISION or ALL_REVISIONS option is selected.
    CurrentCommit,
    /// parse and output all header fields from the output revisions.
    /// If only CURRENT_REVISION was requested then only the current revision’s commit data will be output.
    AllCommits,
    /// list files modified by the commit and magic files, including basic line counts inserted/deleted per file.
    /// Only valid when the CURRENT_REVISION or ALL_REVISIONS option is selected.
    CurrentFiles,
    /// List files modified by the commit and magic files, including basic line counts inserted/deleted
    /// per file. If only the CURRENT_REVISION was requested then only that commit’s modified files will be output.
    AllFiles,
    /// Include _account_id, email and username fields when referencing accounts.
    DetailedAccounts,
    /// Include updates to reviewers set as ReviewerUpdateInfo entities.
    ReviewerUpdates,
    /// Include messages associated with the change.
    Messages,
    /// Include information on available actions for the change and its current revision.
    /// Ignored if the caller is not authenticated.
    CurrentActions,
    /// Include information on available change actions for the change.
    /// Ignored if the caller is not authenticated.
    ChangeActions,
    /// Include the reviewed field if all of the following are true:
    ///  - the change is open
    ///  - the caller is authenticated
    ///  - the caller has commented on the change more recently than the last update from the change owner,
    ///    i.e. this change would show up in the results of reviewedby:self.
    Reviewed,
    /// Skip the 'insertions' and 'deletions' field in ChangeInfo.
    /// For large trees, their computation may be expensive.
    SkipDiffstat,
    /// Include the submittable field in ChangeInfo,
    /// which can be used to tell if the change is reviewed and ready for submit.
    Submittable,
    /// Include the web_links field in CommitInfo, therefore only valid in combination with CURRENT_COMMIT or ALL_COMMITS.
    WebLinks,
    /// Include potential problems with the change.
    Check,
    /// Include the full commit message with Gerrit-specific commit footers in the RevisionInfo.
    CommitFooters,
    /// Include push certificate information in the RevisionInfo. Ignored if signed push is not enabled on the server.
    PushCertificates,
    /// Include references to external tracking systems as TrackingIdInfo.
    TrackingIds,
}

#[derive(Debug)]
pub enum QueryStr {
    Raw(String),
    Cooked(Vec<QueryOpr>),
}

#[derive(Debug)]
pub enum QueryOpr {
    Search(SearchOpr),
    Bool(BoolOpr),
    Group(GroupOpr),
}

#[derive(Debug)]
pub enum SearchOpr {
    Is(Is),
    Owner(String),
    Reviewer(String),
    Limit(u32),
}

#[derive(Debug, AsRefStr, Display, PartialEq, Eq, Clone)]
#[strum(serialize_all = "SCREAMING_SNAKE_CASE")]
pub enum BoolOpr {
    Not,
    And,
    Or,
}

#[derive(Debug, AsRefStr, Display, PartialEq, Eq, Clone)]
pub enum GroupOpr {
    #[strum(serialize = "(")]
    Begin,
    #[strum(serialize = ")")]
    End,
}

#[derive(Debug, AsRefStr, Display, PartialEq, Eq, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
#[strum(serialize_all = "snake_case")]
pub enum Is {
    Assigned,
    Unassigned,
    Starred,
    Watched,
    Reviewed,
    Owner,
    Reviewer,
    Cc,
    Ignored,
    New,
    Open,
    Pending,
    Draft,
    Closed,
    Merged,
    Abandoned,
    Submittable,
    Mergeable,
    Private,
    Wip,
}

impl Serialize for QueryStr {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        match self {
            QueryStr::Raw(s) => serializer.serialize_str(s.as_str()),
            QueryStr::Cooked(operators) => {
                let mut strings: Vec<String> = Vec::new();
                strings.reserve(operators.len());
                for opr in operators {
                    strings.push(format!("{}", opr));
                }
                println!("{:#?}", strings);
                let joined = strings.join(" ");
                serializer.serialize_str(joined.as_str())
            }
        }
    }
}

impl Display for QueryOpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            QueryOpr::Search(s) => write!(f, "{}", s),
            QueryOpr::Bool(b) => write!(f, "{}", b.as_ref()),
            QueryOpr::Group(g) => write!(f, "{}", g.as_ref()),
        }
    }
}

impl Display for SearchOpr {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::result::Result<(), Error> {
        match self {
            SearchOpr::Is(o) => write!(f, "is:{}", o),
            SearchOpr::Owner(o) => write!(f, "owner:{}", o),
            SearchOpr::Reviewer(o) => write!(f, "reviewer:{}", o),
            SearchOpr::Limit(o) => write!(f, "limit:{}", o),
        }
    }
}
