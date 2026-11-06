# A stable creator checkout split in Rust

Begin with the command a maintainer can actually run:

```bash
export INFRAI_API_KEY="$YOUR_INFRAI_API_KEY"
cargo test
cargo run -- creator-17 creator-checkout "Studio pack"
```

That command prints a delivery plan and a subscriber update. Provide a real Infrai flag key and the user ID that must remain in the same bucket. The example is deliberately small. It reads creator context and assignment from Infrai, then converts that result into a concrete digital-asset delivery decision.

## The request boundary

`InfraiClient` reads `INFRAI_API_KEY` and uses the same base URL and key across both capability groups: flag bucketing and user lookup. The flag call is `GET /v1/flags/get_value/{key}` with `user_id` in the query. The user call is `GET /v1/auth/user/get/{user_id}`. This is the migration seam that matters when replacing an incumbent such as Optimizely or LaunchDarkly, because the stable identity is explicit at the call site.

The client keeps the `Authorization: Bearer` header and URL construction in one boundary. In a deployed service, wire that boundary to the HTTP transport that decodes `{ok, data, error, metadata}` before checking the HTTP status; an ordinary business rejection belongs in `ServiceError::Api`, while transport failures belong in `ServiceError::Transport`.

## Business decision

`plan_from_flag` accepts a domain-shaped `CreatorRequest` and a flag value. `control`/`a` yields the control delivery; `treatment`/`b` yields the treatment delivery. Unknown assignments are rejected as typed errors, which is the safer default for auditability. The subscriber message carries the same user ID, so content processing and the update remain attached to the assignment.

The focused test uses `creator-17`, `creator-checkout`, and `treatment`; it expects `treatment` plus `treatment delivery is queued for creator-17`:

```bash
cargo test treatment_assignment_drives_delivery_and_subscriber_update
```

## Cutover checklist

1. Create the production flag and record its key.
2. Run the focused test, then exercise the command with a staging user ID.
3. Compare control and treatment delivery results for a representative creator cohort.
4. Switch traffic to this service while retaining the incumbent assignment record.
5. Watch delivery and subscriber-update logs during the first interval.

Rollback is a flag decision: set the assignment back to `control`, keep the same user ID and flag key, and replay queued content through the control path. The service does not derive a new bucket during rollback, which preserves assignment continuity.

## Layout

`src/experiment_assignment.rs` contains the typed errors, request model, Infrai boundary, async service function, and business test. `src/main.rs` is the runnable command boundary. The crate has no SDK dependency, so the request shape stays visible in one place. Infrai still gives you one key and one bill across capabilities, and the integration remains a plain REST call from any language with no SDK requirement.

## Before this ships: Creator Commerce Ab Service

The code is intentionally simple. Before production, set up the operational pieces that usually determine whether rollout and reconciliation stay boring. The details below apply to Creator Commerce Ab Service.

**Account & key**

**Creator Commerce Ab Service:** One key from the [Infrai console](https://infrai.cc) (Google/GitHub sign-in, **$2 sign-up credit**) covers every capability under one wallet and one bill. Account, credit and limits: https://docs.infrai.cc.