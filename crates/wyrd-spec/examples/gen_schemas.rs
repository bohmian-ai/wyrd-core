//! Generate JSON schema goldens into `crates/wyrd-spec/schemas/` (the single
//! golden set: diffed by `codegen:check` and read by the in-crate drift tests).

use std::fs;
use std::path::Path;

use schemars::schema_for;
use wyrd_spec::auth::{
    AbsoluteUrl, CallbackQuery, IssuerUrl, LoginInitResponse, PrincipalKindTag,
    RevokePrincipalRequest, RevokePrincipalResponse, TokenRequest, TokenResponse,
};
use wyrd_spec::card::agent::AgentSpec;
use wyrd_spec::card::artifact::{ArtifactSpec, FrameworkAdapterRef};
use wyrd_spec::card::audit::AuditSpec;
use wyrd_spec::card::data::{
    DataInterface, DataSchema, DataSpec, DataSplit, DataStats, SplitStrategy, SqlLogic,
};
use wyrd_spec::card::drift::DriftSpec;
use wyrd_spec::card::eval::EvalSpec as CardEvalSpec;
use wyrd_spec::card::experiment::ExperimentSpec;
use wyrd_spec::card::field::FieldSpec;
use wyrd_spec::card::mcp::McpSpec;
use wyrd_spec::card::model::{
    HuggingFaceTask, ModelInterface, ModelSignature, ModelSpec, SampleInput, SampleInputKind,
    TaskType, TfSaveFormat, TorchSaveFormat,
};
use wyrd_spec::card::operator::{
    HttpAuth, HttpMethod, NotifyChannel, OperatorAction, OperatorBudget, OperatorSpec,
};
use wyrd_spec::card::policy::{InvokeContext, InvokeOutcome, PolicyDecision, PolicySpec};
use wyrd_spec::card::prompt::{ParameterName, PromptRef, PromptSpec};
use wyrd_spec::card::service::{LockedComponent, ServiceLock};
use wyrd_spec::card::service::{
    ServiceRuntime, ServiceRuntimeKind, ServiceRuntimeMode, ServiceRuntimePolicy, ServiceSpec,
};
use wyrd_spec::card::source::{
    LogConnection, MetricsConnection, SourceAuth, SourceKind, SourceSpec, SqlConnection,
    TraceConnection,
};
use wyrd_spec::card::trigger::{TriggerSchedule, TriggerSource, TriggerSpec};
use wyrd_spec::card::workflow::WorkflowSpec;
use wyrd_spec::envelope::{Card, CardKind};
use wyrd_spec::reference::CardRef;
use wyrd_spec::registry::{
    ArtifactInventoryResponse, ArtifactManifestEntry, CardLifecycleStatus, CardLocator,
    CardRegistrationOutcome, CardSubmission, CardSummary, CardUploadEntry, CardUploadPlan,
    CreateCardRequest, CreateCardResponse, DeleteCardResponse, GetCardResponse, ListCardsRequest,
    ListCardsResponse, ListVersionsResponse, RegistrationOperationId, RegistrationOutcomeKind,
    RegistrationReplaySeed, RelativeArtifactPath, StoredArtifactEntry,
};
use wyrd_spec::run::{RunKind, RunRef};
use wyrd_spec::security::{SecretRef, TlsConfig};
use wyrd_spec::storage::{
    AbortResponse, DownloadInitRequest, DownloadInitResponse, DownloadPlan,
    LocalBlobUploadResponse, PartUrlResponse, UploadCompleteRequest, UploadCompleteResponse,
    UploadInitRequest, UploadInitResponse, UploadPlan, VerificationGuarantee, WireProtocol,
};
use wyrd_spec::vala::api::{
    AuditDecision, AuditEvent, AuditResult, AuthMethod, BifrostErrorDescriptor,
    BifrostPermissionDescriptor, BifrostQueryRequest, BifrostTableDescription, BifrostTableEntry,
    DataTypeSpec, FieldSpec as BifrostFieldSpec, PartitionColumnSpec, PartitionTransformWire,
    QueryParam, RegisterOutcome, RegisterTableRequest, RegisterTableResponse, SyncQueryRequest,
    TableStatus, TimeUnit,
};
use wyrd_spec::vala::eval::{
    AgentTurnSubmission, ComparisonOperator, ConversationTurn, DagError, EvalCondition,
    EvalPassGate, EvalRecordObservation, EvalRunOpenRequest, EvalRunOpenResponse, EvalSampling,
    EvalScenarioCollection, EvalSpec, EvalTask, ExecutionPlan, SimulatedUserMode,
    SimulatedUserTurn, TurnDirective, UserTurnSubmission,
};
use wyrd_spec::vala::observation::{ObservationEnvelope, ObservationKind, RecordObservation};
use wyrd_spec::vala::trace::{
    AttributeValue, GenAiEvalResult, GenAiSpanRecord, InstrumentationScope, Resource, SpanEvent,
    SpanKind, SpanLink, SpanRecord, SpanStatus, TraceSummaryRecord,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let out = Path::new("crates/wyrd-spec/schemas");
    fs::create_dir_all(out)?;

    write::<Card>(out, "card")?;
    write::<CardKind>(out, "card_kind")?;
    write::<CardRef>(out, "card_ref")?;
    write::<RunKind>(out, "run_kind")?;
    write::<RunRef>(out, "run_ref")?;
    write::<ServiceLock>(out, "service_lock")?;
    write::<LockedComponent>(out, "locked_component")?;
    write::<FieldSpec>(out, "field_spec")?;
    write::<DataSchema>(out, "data_schema")?;
    write::<SplitStrategy>(out, "split_strategy")?;
    write::<DataSplit>(out, "data_split")?;
    write::<DataInterface>(out, "data_interface")?;
    write::<SqlLogic>(out, "sql_logic")?;
    write::<DataStats>(out, "data_stats")?;
    write::<DataSpec>(out, "data_spec")?;
    write::<ModelSpec>(out, "model_spec")?;
    write::<ModelInterface>(out, "model_interface")?;
    write::<TaskType>(out, "task_type")?;
    write::<ModelSignature>(out, "model_signature")?;
    write::<SampleInput>(out, "sample_input")?;
    write::<SampleInputKind>(out, "sample_input_kind")?;
    write::<TorchSaveFormat>(out, "torch_save_format")?;
    write::<TfSaveFormat>(out, "tf_save_format")?;
    write::<HuggingFaceTask>(out, "hugging_face_task")?;
    write::<ExperimentSpec>(out, "experiment_spec")?;
    write::<PromptSpec>(out, "prompt_spec")?;
    write::<PromptRef>(out, "prompt_ref")?;
    write::<ParameterName>(out, "parameter_name")?;
    write::<AgentSpec>(out, "agent_spec")?;
    write::<WorkflowSpec>(out, "workflow_spec")?;
    write::<CardEvalSpec>(out, "eval_spec")?;
    write::<DriftSpec>(out, "drift_spec")?;
    write::<TriggerSpec>(out, "trigger_spec")?;
    write::<TriggerSchedule>(out, "trigger_schedule")?;
    write::<TriggerSource>(out, "trigger_source")?;
    write::<OperatorSpec>(out, "operator_spec")?;
    write::<OperatorAction>(out, "operator_action")?;
    write::<NotifyChannel>(out, "notify_channel")?;
    write::<HttpMethod>(out, "http_method")?;
    write::<HttpAuth>(out, "http_auth")?;
    write::<OperatorBudget>(out, "operator_budget")?;
    write::<SourceSpec>(out, "source_spec")?;
    write::<SourceKind>(out, "source_kind")?;
    write::<SqlConnection>(out, "sql_connection")?;
    write::<MetricsConnection>(out, "metrics_connection")?;
    write::<LogConnection>(out, "log_connection")?;
    write::<TraceConnection>(out, "trace_connection")?;
    write::<SourceAuth>(out, "source_auth")?;
    write::<ServiceSpec>(out, "service_spec")?;
    write::<ServiceRuntime>(out, "service_runtime")?;
    write::<ServiceRuntimeKind>(out, "service_runtime_kind")?;
    write::<ServiceRuntimeMode>(out, "service_runtime_mode")?;
    write::<ServiceRuntimePolicy>(out, "service_runtime_policy")?;
    write::<PolicySpec>(out, "policy_spec")?;
    write::<InvokeContext>(out, "invoke_context")?;
    write::<InvokeOutcome>(out, "invoke_outcome")?;
    write::<PolicyDecision>(out, "policy_decision")?;
    write::<McpSpec>(out, "mcp_spec")?;
    write::<AuditSpec>(out, "audit_spec")?;
    write::<ArtifactSpec>(out, "artifact_spec")?;
    write::<FrameworkAdapterRef>(out, "framework_adapter_ref")?;
    write::<UploadInitRequest>(out, "upload_init_request")?;
    write::<UploadInitResponse>(out, "upload_init_response")?;
    write::<UploadPlan>(out, "upload_plan")?;
    write::<UploadCompleteRequest>(out, "upload_complete_request")?;
    write::<UploadCompleteResponse>(out, "upload_complete_response")?;
    write::<PartUrlResponse>(out, "part_url_response")?;
    write::<AbortResponse>(out, "abort_response")?;
    write::<LocalBlobUploadResponse>(out, "local_blob_upload_response")?;
    write::<DownloadInitRequest>(out, "download_init_request")?;
    write::<DownloadInitResponse>(out, "download_init_response")?;
    write::<DownloadPlan>(out, "download_plan")?;
    write::<WireProtocol>(out, "wire_protocol")?;
    write::<VerificationGuarantee>(out, "verification_guarantee")?;

    // Card registration wire contracts (task 01).
    write::<CardSubmission>(out, "card_submission")?;
    write::<ArtifactManifestEntry>(out, "artifact_manifest_entry")?;
    write::<CardRegistrationOutcome>(out, "card_registration_outcome")?;
    write::<CardUploadPlan>(out, "card_upload_plan")?;
    write::<CardUploadEntry>(out, "card_upload_entry")?;
    write::<CreateCardRequest>(out, "create_card_request")?;
    write::<CreateCardResponse>(out, "create_card_response")?;
    write::<RegistrationReplaySeed>(out, "registration_replay_seed")?;
    write::<RegistrationOperationId>(out, "registration_operation_id")?;
    write::<RelativeArtifactPath>(out, "relative_artifact_path")?;
    write::<CardLifecycleStatus>(out, "card_lifecycle_status")?;
    write::<RegistrationOutcomeKind>(out, "registration_outcome_kind")?;
    write::<GetCardResponse>(out, "get_card_response")?;
    write::<DeleteCardResponse>(out, "delete_card_response")?;
    write::<CardSummary>(out, "card_summary")?;
    write::<ListCardsRequest>(out, "list_cards_request")?;
    write::<ListCardsResponse>(out, "list_cards_response")?;
    write::<ListVersionsResponse>(out, "list_versions_response")?;
    write::<CardLocator>(out, "card_locator")?;
    write::<ArtifactInventoryResponse>(out, "artifact_inventory_response")?;
    write::<StoredArtifactEntry>(out, "stored_artifact_entry")?;

    // Auth contracts.
    write::<TokenRequest>(out, "auth_token_request")?;
    write::<TokenResponse>(out, "auth_token_response")?;
    write::<AbsoluteUrl>(out, "auth_url")?;
    write::<IssuerUrl>(out, "auth_issuer_url")?;
    write::<LoginInitResponse>(out, "auth_login_init_response")?;
    write::<CallbackQuery>(out, "auth_callback_query")?;
    write::<PrincipalKindTag>(out, "auth_principal_kind")?;
    write::<RevokePrincipalRequest>(out, "auth_revoke_principal_request")?;
    write::<RevokePrincipalResponse>(out, "auth_revoke_principal_response")?;

    // Phase 4 section 16: shared security primitives.
    write::<SecretRef>(out, "security_secret_ref")?;
    write::<TlsConfig>(out, "security_tls_config")?;

    // Stage 3 C2a: Bifrost wire contract (table management + query).
    write::<BifrostTableEntry>(out, "bifrost_table_entry")?;
    write::<BifrostTableDescription>(out, "bifrost_table_description")?;
    write::<TableStatus>(out, "bifrost_table_status")?;
    write::<DataTypeSpec>(out, "bifrost_data_type_spec")?;
    write::<BifrostFieldSpec>(out, "bifrost_field_spec")?;
    write::<TimeUnit>(out, "bifrost_time_unit")?;
    write::<PartitionTransformWire>(out, "bifrost_partition_transform")?;
    write::<PartitionColumnSpec>(out, "bifrost_partition_column_spec")?;
    write::<RegisterTableRequest>(out, "bifrost_register_table_request")?;
    write::<RegisterOutcome>(out, "bifrost_register_outcome")?;
    write::<RegisterTableResponse>(out, "bifrost_register_table_response")?;
    write::<QueryParam>(out, "bifrost_query_param")?;
    write::<BifrostQueryRequest>(out, "bifrost_query_request")?;
    write::<SyncQueryRequest>(out, "bifrost_sync_query_request")?;
    write::<AuditEvent>(out, "bifrost_audit_event")?;
    write::<AuthMethod>(out, "bifrost_audit_auth_method")?;
    write::<AuditDecision>(out, "bifrost_audit_decision")?;
    write::<AuditResult>(out, "bifrost_audit_result")?;
    // Stage 3 C7: Bifrost capability catalog — error descriptors + permission descriptors.
    // One source (these types in vala::api), two consumers: gen_schemas (snapshot) +
    // bifrost.list_errors / bifrost.list_permissions MCP tools. Schema drift detected here.
    write::<BifrostErrorDescriptor>(out, "bifrost_error_descriptor")?;
    write::<BifrostPermissionDescriptor>(out, "bifrost_permission_descriptor")?;
    let eval_fixtures = Path::new("crates/wyrd-spec/tests/fixtures/eval/schemas");
    fs::create_dir_all(eval_fixtures)?;
    write_fixture::<EvalSpec>(eval_fixtures, "eval_spec")?;
    write_fixture::<EvalTask>(eval_fixtures, "eval_task")?;
    write_fixture::<EvalCondition>(eval_fixtures, "eval_condition")?;
    write_fixture::<ComparisonOperator>(eval_fixtures, "comparison_operator")?;
    write_fixture::<ExecutionPlan>(eval_fixtures, "execution_plan")?;
    write_fixture::<DagError>(eval_fixtures, "dag_error")?;
    write_fixture::<EvalRecordObservation>(eval_fixtures, "eval_record_observation")?;
    write_fixture::<EvalScenarioCollection>(eval_fixtures, "eval_scenario_collection")?;
    write_fixture::<EvalPassGate>(eval_fixtures, "eval_pass_gate")?;
    write_fixture::<EvalSampling>(eval_fixtures, "eval_sampling")?;
    write_fixture::<EvalRunOpenRequest>(eval_fixtures, "eval_run_open_request")?;
    write_fixture::<EvalRunOpenResponse>(eval_fixtures, "eval_run_open_response")?;
    write_fixture::<SimulatedUserMode>(eval_fixtures, "simulated_user_mode")?;
    write_fixture::<TurnDirective>(eval_fixtures, "turn_directive")?;
    write_fixture::<ConversationTurn>(eval_fixtures, "conversation_turn")?;
    write_fixture::<AgentTurnSubmission>(eval_fixtures, "agent_turn_submission")?;
    write_fixture::<UserTurnSubmission>(eval_fixtures, "user_turn_submission")?;
    write_fixture::<SimulatedUserTurn>(eval_fixtures, "simulated_user_turn")?;

    let trace_fixtures = Path::new("crates/wyrd-spec/tests/fixtures/trace/schemas");
    fs::create_dir_all(trace_fixtures)?;
    write_fixture::<SpanRecord>(trace_fixtures, "span_record")?;
    write_fixture::<SpanKind>(trace_fixtures, "span_kind")?;
    write_fixture::<SpanStatus>(trace_fixtures, "span_status")?;
    write_fixture::<SpanEvent>(trace_fixtures, "span_event")?;
    write_fixture::<SpanLink>(trace_fixtures, "span_link")?;
    write_fixture::<Resource>(trace_fixtures, "resource")?;
    write_fixture::<InstrumentationScope>(trace_fixtures, "instrumentation_scope")?;
    write_fixture::<TraceSummaryRecord>(trace_fixtures, "trace_summary_record")?;
    write_fixture::<GenAiSpanRecord>(trace_fixtures, "gen_ai_span_record")?;
    write_fixture::<GenAiEvalResult>(trace_fixtures, "gen_ai_eval_result")?;
    write_fixture::<AttributeValue>(trace_fixtures, "attribute_value")?;

    let obs_fixtures = Path::new("crates/wyrd-spec/tests/fixtures/observation/schemas");
    fs::create_dir_all(obs_fixtures)?;
    write_fixture::<ObservationEnvelope>(obs_fixtures, "observation_envelope")?;
    write_fixture::<ObservationKind>(obs_fixtures, "observation_kind")?;
    write_fixture::<RecordObservation>(obs_fixtures, "record_observation")?;
    Ok(())
}

fn write<T: schemars::JsonSchema>(
    out: &Path,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut schema = schema_for!(T);
    schema.meta_schema = Some("https://json-schema.org/draft/2020-12/schema".to_string());
    let json = serde_json::to_string_pretty(&schema)?;
    fs::write(out.join(format!("{name}.json")), format!("{json}\n"))?;
    Ok(())
}

fn write_fixture<T: schemars::JsonSchema>(
    dir: &Path,
    name: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut schema = schema_for!(T);
    schema.meta_schema = Some("https://json-schema.org/draft/2020-12/schema".to_string());
    let json = serde_json::to_string_pretty(&schema)?;
    fs::write(dir.join(format!("{name}.schema.json")), format!("{json}\n"))?;
    Ok(())
}
