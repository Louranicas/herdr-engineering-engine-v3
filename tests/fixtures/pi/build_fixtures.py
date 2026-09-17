#!/usr/bin/env python3
"""Author deterministic, fictional Pi 0.85.1 development wire fixtures.

This is fixture authoring, never a Pi launcher or parser oracle. Expectations
come from the pinned emitter/types and explicit worker policy, not Rust output.
"""

from hashlib import sha256
import json
from pathlib import Path


ROOT = Path(__file__).resolve().parent
PACKAGE = Path('/var/home/Louranicas/.local/lib/node_modules/@earendil-works/pi-coding-agent')
SESSION = 'fixture-session-001'
MODEL = {
    'id': 'fixture-model', 'name': 'Fictional fixture model',
    'api': 'openai-completions', 'provider': 'fixture-provider',
    'baseUrl': 'https://fixture.invalid/v1', 'reasoning': True, 'input': ['text'],
    'cost': {'input': 0, 'output': 0, 'cacheRead': 0, 'cacheWrite': 0},
    'contextWindow': 32768, 'maxTokens': 4096,
}
UNAVAILABLE_MODEL = {
    'id': 'unknown', 'name': 'unknown', 'api': 'unknown', 'provider': 'unknown',
    'baseUrl': '', 'reasoning': False, 'input': [],
    'cost': {'input': 0, 'output': 0, 'cacheRead': 0, 'cacheWrite': 0},
    'contextWindow': 0, 'maxTokens': 0,
}
USAGE = {
    'input': 20, 'output': 10, 'cacheRead': 3, 'cacheWrite': 6,
    'cacheWrite1h': 2, 'reasoning': 4, 'totalTokens': 39,
    'cost': {'input': 0.02, 'output': 0.01, 'cacheRead': 0.003,
             'cacheWrite': 0.006, 'total': 0.039},
}
ZERO_USAGE = {
    'input': 0, 'output': 0, 'cacheRead': 0, 'cacheWrite': 0,
    'totalTokens': 0,
    'cost': {'input': 0, 'output': 0, 'cacheRead': 0, 'cacheWrite': 0, 'total': 0},
}
USER = {'role': 'user', 'content': 'Fictional prompt; do not execute.', 'timestamp': 1000}
ASSISTANT = {
    'role': 'assistant', 'content': [{'type': 'text', 'text': 'final Y'}],
    'api': MODEL['api'], 'provider': MODEL['provider'], 'model': MODEL['id'],
    'usage': USAGE, 'stopReason': 'stop', 'timestamp': 1001,
}
PARTIAL = {**ASSISTANT, 'content': [], 'usage': ZERO_USAGE, 'stopReason': 'pending'}
ERROR_MESSAGE = {
    **ASSISTANT, 'content': [], 'usage': ZERO_USAGE, 'stopReason': 'error',
    'errorMessage': 'Fictional overloaded provider',
}
STATE = {
    'thinkingLevel': 'off', 'isStreaming': False, 'isCompacting': False,
    'steeringMode': 'all', 'followUpMode': 'one-at-a-time', 'sessionId': SESSION,
    'autoCompactionEnabled': False, 'messageCount': 0, 'pendingMessageCount': 0,
}
STATS = {
    'sessionId': SESSION, 'userMessages': 1, 'assistantMessages': 1,
    'toolCalls': 0, 'toolResults': 0, 'totalMessages': 2,
    'tokens': {'input': 20, 'output': 10, 'cacheRead': 3, 'cacheWrite': 6, 'total': 39},
    'cost': 0.039,
}
SOURCES = {
    'package': ('package.json', ['1-106']),
    'rpc': ('dist/modes/rpc/rpc-mode.js', ['21-40', '292-397', '468-472', '604-654']),
    'rpc_types': ('dist/modes/rpc/rpc-types.d.ts', ['17-63', '149-235', '280-311']),
    'json_event': ('dist/modes/json-event.js', ['1-31']),
    'jsonl': ('dist/modes/rpc/jsonl.js', ['1-61']),
    'session': ('dist/core/agent-session.js', ['309-440', '616-623', '765-810', '821-953',
                                            '1190-1234', '1356-1378', '2283-2334', '2651-2748']),
    'session_types': ('dist/core/agent-session.d.ts', ['38-96', '174-195']),
    'agent_types': ('node_modules/@earendil-works/pi-agent-core/dist/types.d.ts', ['260-287', '367-421']),
    'default_model': ('node_modules/@earendil-works/pi-agent-core/dist/agent.js', ['14-34']),
    'ai_types': ('node_modules/@earendil-works/pi-ai/dist/types.d.ts', ['235-353', '398-469', '695-738']),
}


def wire(value):
    return (json.dumps(value, ensure_ascii=False, separators=(',', ':'), allow_nan=False) + '\n').encode('utf-8')


def identity(data):
    return {'byte_length': len(data), 'sha256': sha256(data).hexdigest()}


def response(command, data=None, *, request_id=None, error=None):
    value = {'id': request_id or 'fixture-' + command, 'type': 'response',
             'command': command, 'success': error is None}
    if error is not None:
        value['error'] = error
    elif data is not None:
        value['data'] = data
    return value


def update(event, usage=None):
    return {'type': 'message_update', 'usage': ZERO_USAGE if usage is None else usage,
            'assistantMessageEvent': event}


def main():
    records = []
    by_id = {}
    values = {}

    def record(case_id, value, *, accept=True, layer='record', kind=None,
               meaning='', context=None, sources=('rpc', 'rpc_types'), raw=None):
        data = wire(value) if raw is None else raw
        path = f'records/{case_id}.jsonl' if raw is None else f'records/{case_id}.wire'
        (ROOT / path).parent.mkdir(parents=True, exist_ok=True)
        (ROOT / path).write_bytes(data)
        item = {'id': case_id, 'path': path, **identity(data), 'layer': layer,
                'expected_accept': accept, 'expected_kind': kind,
                'expected_meaning': meaning, 'context': context or {}, 'source_ids': list(sources)}
        if isinstance(value, dict) and value.get('type') == 'response' and 'id' in value:
            item['context'].setdefault('pending', {'id': value['id'], 'command': value['command']})
        records.append(item)
        by_id[case_id] = item
        values[case_id] = value
        return case_id

    # The admitted command vocabulary. These are input fixtures, never launch instructions.
    for command, extra in [
        ('get_state', {}), ('get_available_models', {}),
        ('set_model', {'provider': MODEL['provider'], 'modelId': MODEL['id']}),
        ('set_thinking_level', {'level': 'high'}),
        ('prompt', {'message': USER['content']}), ('clear_queue', {}), ('abort', {}),
        ('get_session_stats', {}),
    ]:
        record('request-' + command, {'id': 'fixture-' + command, 'type': command, **extra},
               layer='outbound_profile', kind='request',
               meaning='Closed profile request; fixture bytes do not authorize dispatch.')
    record('request-prompt-images', {'id': 'fixture-prompt', 'type': 'prompt',
           'message': USER['content'], 'images': []}, accept=False, layer='outbound_profile',
           meaning='Vendor option is outside this text-only profile.')
    record('request-bash', {'id': 'fixture-bash', 'type': 'bash', 'command': 'true'},
           accept=False, layer='outbound_profile', meaning='Unsupported command; no dispatch.')

    record('state-empty', response('get_state', STATE), kind='state',
           meaning='Allowed optional-model type shape; model unavailable. Actual empty SDK capture uses explicit unknown sentinel instead.',
           context={'owned_session_id': SESSION})
    record('state-unavailable-sentinel', response('get_state', {**STATE, 'model': UNAVAILABLE_MODEL}),
           kind='state', meaning='Exact SDK DEFAULT_MODEL sentinel is unavailable, never an effective routable model.',
           sources=('rpc', 'default_model', 'metadata_capture'))
    for case_id, model in [
        ('sentinel-with-route', {**UNAVAILABLE_MODEL, 'baseUrl': 'https://fixture.invalid/v1'}),
        ('sentinel-with-cost', {**UNAVAILABLE_MODEL, 'cost': {**UNAVAILABLE_MODEL['cost'], 'input': 1}}),
        ('sentinel-with-window', {**UNAVAILABLE_MODEL, 'contextWindow': 1}),
        ('sentinel-with-name', {**UNAVAILABLE_MODEL, 'name': 'Fictional route'}),
    ]:
        record(case_id, response('get_state', {**STATE, 'model': model}), accept=False,
               meaning='Reserved unknown sentinel must match exact DEFAULT_MODEL; near-neighbor is neither valid sentinel nor selected model.',
               sources=('rpc', 'default_model'))
    model_state = {**STATE, 'model': MODEL, 'thinkingLevel': 'high'}
    record('state-model', response('get_state', model_state), kind='state',
           meaning='Effective fictional provider/model and high thinking, subject to recipe comparison.',
           context={'owned_session_id': SESSION})
    record('state-after-run', response('get_state', {**model_state, 'messageCount': 2}), kind='state',
           meaning='Synthetic final state for one user/assistant pair; idle and queues empty.',
           context={'owned_session_id': SESSION})
    record('state-foreign-model', response('get_state', {**model_state, 'model': {**MODEL, 'id': 'fixture-other-model'}}),
           kind='state', meaning='Structurally valid different model; recipe identity comparison must refuse mismatch.')
    record('state-compacting', response('get_state', {**STATE, 'isCompacting': True}), kind='state',
           meaning='Structurally valid observation; not idle. Unexpected compaction blocks this profile.')
    record('state-queued', response('get_state', {**STATE, 'pendingMessageCount': 1}), kind='state',
           meaning='Structurally valid observation; not idle even with both activity flags false.')
    record('models-empty', response('get_available_models', {'models': []}), kind='available_models',
           meaning='No available model; metadata-only result, not an error or prompt grant.')
    record('models-one', response('get_available_models', {'models': [MODEL]}), kind='available_models',
           meaning='One fictional model descriptor; no real configured provider is asserted.')
    record('model-image-capability', response('set_model', {**MODEL, 'input': ['text', 'image']}),
           kind='model', meaning='Image capability is valid metadata; this does not admit prompt image payloads.')
    record('model-selected', response('set_model', MODEL), kind='model',
           meaning='Set response only; final correlated get_state must match recipe.')
    record('model-unavailable', response('set_model', error='Model not found: fixture-provider/fixture-model'),
           kind='command_error', meaning='Correlated refusal; no selected model.')
    record('thinking-ack', response('set_thinking_level'), kind='ack',
           meaning='Acknowledgement can follow clamping; not proof of requested effective level.')
    record('prompt-ack', response('prompt'), kind='ack',
           meaning='Preflight acknowledgement only; execution, candidate, Pi settlement and acceptance all unproved.')
    record('prompt-refused', response('prompt', error='Fictional missing credentials'), kind='command_error',
           meaning='Correlated preflight refusal; never infer an executed attempt.')
    record('clear-removed', response('clear_queue', {'steering': ['old steering'], 'followUp': ['old follow-up']}),
           kind='cleared_queue', meaning='Arrays are removed prior contents, not remaining queue counts.', sources=('rpc', 'session'))
    record('clear-empty', response('clear_queue', {'steering': [], 'followUp': []}),
           kind='cleared_queue', meaning='Both prior queues were empty.', sources=('rpc', 'session'))
    record('abort-ack', response('abort'), kind='ack',
           meaning='Pi abort awaited idle; own state readback and external custody remain required.')
    record('stats-known', response('get_session_stats', STATS), kind='session_stats',
           meaning='Session total tokens=20+10+3+6=39; costs are fictional provider-reported numbers.',
           context={'owned_session_id': SESSION}, sources=('rpc', 'session', 'session_types'))
    record('stats-unknown-context', response('get_session_stats', {**STATS, 'contextUsage': {
        'tokens': None, 'contextWindow': 32768, 'percent': None}}), kind='session_stats',
        meaning='Preserve unknown tokens/percent as null, not zero.', sources=('session', 'session_types'))
    record('stats-known-context', response('get_session_stats', {**STATS, 'contextUsage': {
        'tokens': 1024, 'contextWindow': 32768, 'percent': 3.125}}), kind='session_stats',
        meaning='1024/32768*100=3.125; context snapshot does not replace session totals.', sources=('session', 'session_types'))

    # All events below are structurally tested without claiming standalone legal ordering.
    event_sources = ('json_event', 'session', 'session_types', 'agent_types', 'ai_types')
    for case_id, value, kind, meaning in [
        ('agent-start', {'type': 'agent_start'}, 'agent_start', 'Activity began; original deadline remains unchanged.'),
        ('turn-start', {'type': 'turn_start'}, 'turn_start', 'Turn observation, not settlement.'),
        ('user-start', {'type': 'message_start', 'message': USER}, 'message_start', 'User text remains data.'),
        ('user-end', {'type': 'message_end', 'message': USER}, 'message_end', 'Complete user message.'),
        ('assistant-start', {'type': 'message_start', 'message': PARTIAL}, 'message_start', 'Pending assistant; zero usage not final accounting.'),
        ('update-start', update({'type': 'start'}), 'message_update', 'Wire partial field omitted.'),
        ('text-start', update({'type': 'text_start', 'contentIndex': 0}), 'message_update', 'Text block begins at index zero.'),
        ('text-delta', update({'type': 'text_delta', 'contentIndex': 0, 'delta': 'draft X'}), 'message_update', 'Presentation delta; never execute text.'),
        ('text-end', update({'type': 'text_end', 'contentIndex': 0, 'content': 'final Y'}, USAGE), 'message_update', 'Authoritative block end may differ from deltas.'),
        ('thinking-start', update({'type': 'thinking_start', 'contentIndex': 0}), 'message_update', 'Known thinking event; no activity renewal.'),
        ('thinking-delta', update({'type': 'thinking_delta', 'contentIndex': 0, 'delta': 'fixture reasoning'}), 'message_update', 'Opaque model output; no control authority.'),
        ('thinking-end', update({'type': 'thinking_end', 'contentIndex': 0, 'content': 'fixture reasoning'}), 'message_update', 'Known thinking block closure.'),
        ('update-done', update({'type': 'done', 'reason': 'stop', 'message': ASSISTANT}, USAGE), 'message_update', 'Actual emitter keeps done.message; not Pi settled.'),
        ('update-error', update({'type': 'error', 'reason': 'error', 'error': ERROR_MESSAGE}), 'message_update', 'Error event retains full assistant error; not prompt preflight failure.'),
        ('assistant-end', {'type': 'message_end', 'message': ASSISTANT}, 'message_end', 'Final text final Y is authoritative; preserve usage subsets without double-add.'),
        ('assistant-error-end', {'type': 'message_end', 'message': ERROR_MESSAGE}, 'message_end', 'Completed error message; retry remains possible.'),
        ('turn-end', {'type': 'turn_end', 'message': ASSISTANT, 'toolResults': []}, 'turn_end', 'No-tools turn; still not settled.'),
        ('agent-end-no-retry', {'type': 'agent_end', 'messages': [USER, ASSISTANT], 'willRetry': False}, 'agent_end', 'False willRetry does not rule out compaction or queued continuation.'),
        ('agent-end-continuation', {'type': 'agent_end', 'messages': [ASSISTANT], 'willRetry': False}, 'agent_end', 'Continuation produced one assistant message; still await agent_settled.'),
        ('agent-end-retry', {'type': 'agent_end', 'messages': [ERROR_MESSAGE], 'willRetry': True}, 'agent_end', 'Retry indicated; remain active.'),
        ('retry-start', {'type': 'auto_retry_start', 'attempt': 1, 'maxAttempts': 3, 'delayMs': 2000,
                         'errorMessage': ERROR_MESSAGE['errorMessage']}, 'auto_retry_start', 'Backoff does not renew worker deadline.'),
        ('retry-success', {'type': 'auto_retry_end', 'success': True, 'attempt': 1}, 'auto_retry_end', 'Successful retried message observed; still await final settlement.'),
        ('retry-cancelled', {'type': 'auto_retry_end', 'success': False, 'attempt': 1,
                             'finalError': 'Retry cancelled'}, 'auto_retry_end', 'Retry sleep cancelled; not external custody settlement.'),
        ('agent-settled', {'type': 'agent_settled'}, 'agent_settled', 'Only Pi continuation settlement; require owned lifecycle and idle readback.'),
        ('queue-empty', {'type': 'queue_update', 'steering': [], 'followUp': []}, 'queue_update', 'Empty current queues; not request response.'),
        ('queue-nonempty', {'type': 'queue_update', 'steering': ['old steering'], 'followUp': ['old follow-up']}, 'queue_update', 'Unexpected queued work remains explicit; not permission to execute it.'),
        ('thinking-changed', {'type': 'thinking_level_changed', 'level': 'high'}, 'thinking_level_changed', 'Effective level event precedes set_thinking response when changed.'),
    ]:
        record(case_id, value, kind=kind, meaning=meaning, sources=event_sources)

    # Specific reviewed structural faults; none are generated by inspecting Rust code.
    faults = [
        ('state-missing-field', response('get_state', {k: v for k, v in STATE.items() if k != 'isStreaming'}), 'Required state field missing.'),
        ('state-unknown-field', response('get_state', {**STATE, 'settled': True}), 'Unknown nested state field cannot supply settlement.'),
        ('state-null-model', response('get_state', {**STATE, 'model': None}), 'Undefined model is omitted by emitter; null is a different invalid type.'),
        ('state-bad-enum', response('get_state', {**STATE, 'thinkingLevel': 'ultra'}), 'Unknown thinking level.'),
        ('state-bool-counter', response('get_state', {**STATE, 'messageCount': True}), 'Boolean is not a numeric counter.'),
        ('state-negative-counter', response('get_state', {**STATE, 'messageCount': -1}), 'Worker counter policy rejects negative values.'),
        ('state-fractional-counter', response('get_state', {**STATE, 'messageCount': 1.5}), 'Worker counter policy rejects fractional values.'),
        ('state-unsafe-counter', response('get_state', {**STATE, 'messageCount': 9007199254740992}), 'Above JS safe exact integer ceiling.'),
        ('model-missing-cost', response('set_model', {k: v for k, v in MODEL.items() if k != 'cost'}), 'Required Model cost missing.'),
        ('model-unknown-cost', response('set_model', {**MODEL, 'cost': {**MODEL['cost'], 'currency': 'USD'}}), 'Unreviewed currency field; cannot invent billing currency.'),
        ('model-bad-input', response('set_model', {**MODEL, 'input': ['audio']}), 'Unsupported upstream input modality enum.'),
        ('model-open-headers', response('set_model', {**MODEL, 'headers': {'X-Fixture': 'value'}}), 'Known upstream open map is deliberately unadmitted in initial closed profile.'),
        ('stats-missing-total', response('get_session_stats', {**STATS, 'tokens': {k: v for k, v in STATS['tokens'].items() if k != 'total'}}), 'Required stats token total missing.'),
        ('stats-unknown-token', response('get_session_stats', {**STATS, 'tokens': {**STATS['tokens'], 'reasoning': 4}}), 'SessionStats tokens has no reasoning field.'),
        ('ack-null-data', {**response('prompt'), 'data': None}, 'Successful no-data response omits data; null is an extra field.'),
        ('response-success-string', {**response('abort'), 'success': 'true'}, 'Success must be boolean.'),
        ('response-error-and-data', {**response('prompt', error='fixture refusal'), 'data': {}}, 'Failure envelope has error and no data.'),
        ('response-missing-error', {'id': 'fixture-prompt', 'type': 'response', 'command': 'prompt', 'success': False}, 'Failure envelope requires error string.'),
        ('response-unknown-field', {**response('abort'), 'accepted': True}, 'Unknown root field cannot assert HEE acceptance.'),
        ('delta-old-message', {**update({'type': 'text_delta', 'contentIndex': 0, 'delta': 'x'}), 'message': PARTIAL}, 'Pinned wire emitter omits cumulative message.'),
        ('delta-old-partial', update({'type': 'text_delta', 'contentIndex': 0, 'delta': 'x', 'partial': PARTIAL}), 'Pinned wire emitter removes nested partial.'),
        ('delta-missing-usage', {'type': 'message_update', 'assistantMessageEvent': {'type': 'start'}}, 'Wire message_update requires usage.'),
        ('delta-negative-index', update({'type': 'text_start', 'contentIndex': -1}), 'Negative content index violates selected counter profile.'),
        ('delta-unknown-kind', update({'type': 'citation_delta', 'delta': 'fixture'}), 'Unreviewed nested event kind.'),
        ('assistant-unknown-block', {'type': 'message_end', 'message': {**ASSISTANT, 'content': [{'type': 'html', 'text': '<b>x</b>'}]}}, 'Unreviewed content block type.'),
        ('assistant-unknown-usage', {'type': 'message_end', 'message': {**ASSISTANT, 'usage': {**USAGE, 'providerRequests': 1}}}, 'Usage does not report authoritative provider request count.'),
        ('assistant-missing-stop', {'type': 'message_end', 'message': {k: v for k, v in ASSISTANT.items() if k != 'stopReason'}}, 'Required stop reason missing.'),
        ('agent-end-missing-retry', {'type': 'agent_end', 'messages': []}, 'AgentSession wire adds mandatory willRetry.'),
        ('agent-settled-extra', {'type': 'agent_settled', 'accepted': True}, 'Pinned settled event has only type.'),
        ('queue-nonstring', {'type': 'queue_update', 'steering': [1], 'followUp': []}, 'Queue contents are strings.'),
    ]
    for case_id, value, reason in faults:
        record(case_id, value, accept=False, meaning=reason, sources=event_sources + ('rpc_types',))
    record('state-safe-counter', response('get_state', {**STATE, 'messageCount': 9007199254740991}),
           kind='state', meaning='Exactly-at-safe-ceiling counter is structurally acceptable; not a real session-size assertion.')

    # Recognizable vendor event names refused before unreviewed nested projection.
    for case_id, value, reason in [
        ('unsupported-compaction', {'type': 'compaction_start', 'reason': 'threshold'}, 'Compaction branch is not admitted; refusal cannot settle run.'),
        ('unsupported-entry', {'type': 'entry_appended', 'entry': {'type': 'custom', 'id': 'fictional-entry',
             'parentId': None, 'timestamp': '2026-09-15T00:00:00.000Z', 'customType': 'fixture', 'data': {}}}, 'SessionEntry/custom-extension schema has not been admitted.'),
        ('unsupported-tool-start', {'type': 'tool_execution_start', 'toolCallId': 'fixture-tool-1',
             'toolName': 'fixture-tool', 'args': {}}, 'Unexpected tool execution in no-tools profile.'),
        ('unsupported-toolcall-start', update({'type': 'toolcall_start', 'contentIndex': 0,
             'id': 'fixture-tool-1', 'toolName': 'fixture-tool'}), 'Actual transformed toolcall_start fields, but tool branch remains unadmitted.'),
        ('unsupported-toolcall-delta', update({'type': 'toolcall_delta', 'contentIndex': 0, 'delta': '{"x":'}), 'Incomplete argument text is not executable; tool branch unadmitted.'),
        ('unsupported-deferred', {'type': 'message_end', 'message': {**ASSISTANT, 'stopReason': 'deferred',
             'deferred': {'provider': MODEL['provider'], 'modelId': MODEL['id'], 'api': MODEL['api'], 'id': 'fixture-deferred'}}}, 'Deferred effects cannot become accepted output under current command subset.'),
        ('unknown-lifecycle', {'type': 'agent_verified'}, 'Unknown lifecycle name has no authority.'),
    ]:
        record(case_id, value, accept=False, layer='profile', meaning=reason, sources=event_sources)

    # Correlation faults are structurally shaped responses, not schema type faults.
    record('response-wrong-id', response('get_state', STATE, request_id='fixture-stale'), accept=False,
           layer='correlation', context={'pending': {'id': 'fixture-get_state', 'command': 'get_state'}},
           meaning='Wrong pending id; do not advance state.')
    record('response-wrong-command', response('abort', request_id='fixture-get_state'), accept=False,
           layer='correlation', context={'pending': {'id': 'fixture-get_state', 'command': 'get_state'}},
           meaning='Correct id with wrong command; do not advance state.')
    record('response-no-id', {'type': 'response', 'command': 'parse', 'success': False, 'error': 'fixture parse failure'},
           accept=False, layer='correlation', meaning='Vendor parse failure can be uncorrelated; preserve refusal, never attach to a request.')
    record('stats-wrong-session', response('get_session_stats', {**STATS, 'sessionId': 'fixture-foreign-session'}),
           accept=False, layer='attribution', context={'owned_session_id': SESSION},
           meaning='Valid stats shape but foreign session; no usage attribution.')

    ack = wire(response('prompt'))
    raw_cases = [
        ('raw-unterminated', ack[:-1], 'framing', 'Nonempty EOF fragment is truncation.'),
        ('raw-crlf', ack[:-1] + b'\r\n', 'framing', 'Chosen strict worker LF profile refuses CRLF; vendor input reader is more permissive.'),
        ('raw-empty-line', b'\n', 'framing', 'Empty record is not JSON.'),
        ('raw-invalid-utf8', b'{"type":"agent_\xffstart"}\n', 'framing', 'Invalid UTF-8 before JSON projection.'),
        ('raw-duplicate-id', b'{"id":"fixture-prompt","id":"fixture-stale","type":"response","command":"prompt","success":true}\n', 'json', 'Duplicate keys refused; no last-key-wins.'),
        ('raw-duplicate-nested', wire(response('get_state', STATE)).replace(b'"messageCount":0', b'"messageCount":0,"messageCount":1'), 'json', 'Duplicate keys refused at every depth.'),
        ('raw-trailing-value', b'{"type":"agent_start"}{}\n', 'json', 'Exactly one JSON object per LF frame.'),
        ('raw-nonfinite', wire(response('get_state', STATE)).replace(b'"messageCount":0', b'"messageCount":1e309'), 'json', 'Nonfinite counter in otherwise complete state cannot be accepted.'),
        ('raw-array-root', b'[]\n', 'record', 'Root must be a closed protocol object.'),
    ]
    for case_id, data, layer, reason in raw_cases:
        record(case_id, None, raw=data, accept=False, layer=layer, meaning=reason, sources=('jsonl',))
    unicode_value = update({'type': 'text_delta', 'contentIndex': 0, 'delta': 'A\u2028B\u2029Cé🙂'})
    record('unicode-delta', unicode_value, kind='message_update',
           meaning='One LF frame; literal U+2028/U+2029 and multibyte text stay in string.', sources=('jsonl', 'json_event'))
    record('stderr-forged-pass', None, raw=b'PASS\n{"id":"fixture-prompt","type":"response","command":"prompt","success":true}\n',
           layer='stderr', accept=False, meaning='Raw stderr is retained diagnostic evidence; neither text nor JSON is a response.', sources=('rpc', 'jsonl'))

    traces = []

    def trace(trace_id, steps, meaning, final):
        incoming = bytearray()
        mapped_steps = []
        received_index = 0
        for direction, case_id, observation in steps:
            item = {'direction': direction, 'fixture_id': case_id, 'expected_observation': observation}
            if direction == 'receive':
                data = (ROOT / by_id[case_id]['path']).read_bytes()
                item.update({'record_index': received_index, 'byte_offset': len(incoming), 'byte_length': len(data)})
                received_index += 1
                incoming.extend(data)
            mapped_steps.append(item)
        path = f'traces/{trace_id}.jsonl'
        (ROOT / path).parent.mkdir(exist_ok=True)
        (ROOT / path).write_bytes(incoming)
        traces.append({'id': trace_id, 'path': path, **identity(incoming), 'steps': mapped_steps,
                       'expected_meaning': meaning, 'expected_final': final,
                       'source_ids': ['rpc', 'session', 'json_event', 'ai_types']})

    trace('metadata-empty', [
        ('send', 'request-get_state', 'Bind exact pending id+command before write.'),
        ('receive', 'state-unavailable-sentinel', 'Bind fictional session and observe idle/unavailable sentinel.'),
        ('send', 'request-get_available_models', 'Independent metadata request after state response.'),
        ('receive', 'models-empty', 'No model available; no prompt should be sent.'),
    ], 'Credential-free metadata can succeed with an empty model inventory.',
        {'pi_idle': True, 'provider_calls_authorized': False, 'attempt_accepted': False})
    trace('model-thinking-readback', [
        ('send', 'request-get_available_models', 'Bind discovery.'), ('receive', 'models-one', 'Fictional descriptor available.'),
        ('send', 'request-set_model', 'Await correlated model set before thinking command.'), ('receive', 'model-selected', 'Set observed; readback pending.'),
        ('send', 'request-set_thinking_level', 'Requested high.'), ('receive', 'thinking-changed', 'Changed event can precede response.'),
        ('receive', 'thinking-ack', 'Acknowledgement only.'), ('send', 'request-get_state', 'Confirm effective recipe.'),
        ('receive', 'state-model', 'Exact fictional model identity and high thinking match.'),
    ], 'Dependent commands are serialized and final state supplies effective configuration.',
        {'recipe_matches': True, 'attempt_accepted': False})
    trace('thinking-clamped', [
        ('send', 'request-set_thinking_level', 'Requested high.'), ('receive', 'thinking-ack', 'No proof of high.'),
        ('send', 'request-get_state', 'Required readback.'), ('receive', 'state-empty', 'Observed off; recipe mismatch.'),
    ], 'Success acknowledgement cannot override readback mismatch.', {'recipe_matches': False, 'attempt_accepted': False})
    trace('model-readback-mismatch', [
        ('send', 'request-set_model', 'Requested exact fixture-provider/fixture-model.'),
        ('receive', 'model-selected', 'Correlated requested descriptor returned.'),
        ('send', 'request-get_state', 'Required effective model readback.'),
        ('receive', 'state-foreign-model', 'Observed fixture-other-model; refuse recipe compatibility.'),
    ], 'A matching set response cannot override a different effective provider/model in final state.',
        {'recipe_matches': False, 'attempt_accepted': False})
    trace('prompt-ack-only', [
        ('send', 'request-prompt', 'Bind owned run before send.'), ('receive', 'prompt-ack', 'Preflight acknowledged only.'),
    ], 'EOF after only an ack is not a completed execution.',
        {'prompt_acknowledged': True, 'execution_observed': False, 'pi_settled': False, 'attempt_accepted': False})
    normal = [
        ('send', 'request-prompt', 'Prebind owned run and original deadline.'),
        ('receive', 'prompt-ack', 'Preflight only.'), ('receive', 'agent-start', 'Run active.'),
        ('receive', 'turn-start', 'Turn active.'), ('receive', 'user-start', 'User message started.'),
        ('receive', 'user-end', 'User message ended.'), ('receive', 'assistant-start', 'Pending assistant.'),
        ('receive', 'update-start', 'Native partial omitted.'), ('receive', 'text-start', 'Block index0.'),
        ('receive', 'text-delta', 'Provisional draft X.'), ('receive', 'text-end', 'Block final Y.'),
        ('receive', 'update-done', 'Full final message retained.'), ('receive', 'assistant-end', 'Final Y authoritative; tokens39.'),
        ('receive', 'turn-end', 'Turn closed, run not yet settled.'), ('receive', 'agent-end-no-retry', 'Still not settlement.'),
        ('receive', 'agent-settled', 'Owned Pi continuation settled.'),
        ('send', 'request-get_state', 'Idle readback required.'), ('receive', 'state-after-run', 'Idle flags, queues and two observed messages.'),
    ]
    trace('prompt-final-overrides-deltas', normal,
          'Synthetic finite lifecycle: final Y overrides draft X; no HEE task/admission inference.',
          {'final_text': 'final Y', 'reported_total_tokens': 39, 'pi_settled': True,
           'pi_idle': True, 'external_custody': 'unmeasured', 'attempt_accepted': False})
    trace('retry-remains-active', [
        ('send', 'request-prompt', 'Bind one nonrenewable deadline.'), ('receive', 'prompt-ack', 'Preflight.'),
        ('receive', 'agent-start', 'First run active.'), ('receive', 'assistant-error-end', 'Error message observed.'),
        ('receive', 'agent-end-retry', 'Retry indicated; no settlement.'), ('receive', 'retry-start', 'Same deadline through backoff.'),
        ('receive', 'agent-start', 'Continuation active.'), ('receive', 'assistant-end', 'Successful message.'),
        ('receive', 'retry-success', 'Emitted after successful assistant message_end.'),
        ('receive', 'agent-end-continuation', 'Still await settled.'), ('receive', 'agent-settled', 'Pi settled only now.'),
    ], 'Targeted lifecycle subsequence, not a complete transcript or enabled metadata-smoke retry policy.',
        {'pi_settled': True, 'deadline_renewals': 0, 'attempt_accepted': False})
    trace('clear-removed-then-abort', [
        ('send', 'request-clear_queue', 'Cancel an already-owned active run; do not send abort yet.'),
        ('receive', 'queue-empty', 'Current queues empty; response still pending.'),
        ('receive', 'clear-removed', 'Correlated response reports removed strings; permits dependent abort dispatch.'),
        ('send', 'request-abort', 'Now abort with its own bound id.'),
        ('receive', 'agent-settled', 'Owned run settles during abort.'), ('receive', 'abort-ack', 'Pi abort waiter resolved.'),
        ('send', 'request-get_state', 'Read current Pi state.'), ('receive', 'state-empty', 'Idle queues and activity flags.'),
    ], 'Precondition: an owned active run exists; clear_queue arrays are removed contents, not pending work.',
        {'pi_idle': True, 'pi_settled': True, 'external_custody': 'unmeasured', 'attempt_accepted': False})
    trace('cancel-unsafe-dispatch', [
        ('send', 'request-clear_queue', 'Clear pending.'), ('send', 'request-abort', 'Refuse dependent send before correlated clear_queue response.'),
    ], 'A controller must refuse the second outbound step; no fake incoming reply can repair premature dispatch.',
        {'dispatch_rejected_at_step': 1, 'pi_settled': False, 'attempt_accepted': False})
    trace('stale-settled', [('receive', 'agent-settled', 'No bound active run: reject lifecycle attribution.')],
          'A valid event shape alone does not establish an owned run.',
          {'lifecycle_accepted': False, 'pi_settled': False, 'attempt_accepted': False})
    trace('compaction-after-end', [
        ('receive', 'agent-end-no-retry', 'Assume owned active run; willRetry false does not settle it.'),
        ('receive', 'unsupported-compaction', 'Refuse unsupported continuation; retain unresolved run.'),
        ('receive', 'agent-settled', 'After refusal, do not resume authoritative state merely on a later valid event.'),
    ], 'Unsupported continuation is fail-closed; its subsequent settled event cannot launder the refusal.',
        {'profile_accepted': False, 'pi_settled': False, 'attempt_accepted': False})

    # Exact raw-byte recipes avoid storing many megabytes of repeated padding.
    # These exercise only the named layer; e.g. repeated agent_start records are
    # valid framer input and deliberately NOT a legal lifecycle transcript.
    recipes = []

    def recipe(recipe_id, parts, layer, accept, meaning):
        data = bytearray()
        for part in parts:
            unit = (ROOT / by_id[part['fixture_id']]['path']).read_bytes() if 'fixture_id' in part else part['ascii'].encode('ascii')
            data.extend(unit * part.get('repeat', 1))
        recipes.append({'id': recipe_id, 'parts': parts, **identity(data), 'layer': layer,
                        'expected_accept': accept, 'expected_meaning': meaning})

    prefix = '{"id":"fixture-prompt","type":"response","command":"prompt","success":false,"error":"'
    suffix = '"}\n'
    for limit, label, accepted in [(1048576, 'at', True), (1048577, 'over', False)]:
        pad = limit - len(prefix) - (len(suffix) - 1)
        recipe('frame-' + label + '-limit', [{'ascii': prefix}, {'ascii': 'x', 'repeat': pad}, {'ascii': suffix}],
               'framing', accepted, f'{limit} payload bytes before LF; chosen 1MiB cap excludes LF.')
    stream_unit_size = 1048576
    stream_pad = stream_unit_size - len(prefix) - len(suffix)
    # Store the unit compactly as a composition, not one giant manifest string.
    for extra, label, accepted in [(0, 'at', True), (1, 'over', False)]:
        parts = []
        for _ in range(8):
            parts.extend([{'ascii': prefix}, {'ascii': 'x', 'repeat': stream_pad}, {'ascii': suffix}])
        if extra:
            parts.append({'ascii': 'x'})
        recipe('stream-' + label + '-limit', parts, 'stream_bound', accepted,
               '8MiB cumulative raw stream cap includes delimiters; response correlation is deliberately outside this test.')
    for count, label, accepted in [(65536, 'at', True), (65537, 'over', False)]:
        recipe('records-' + label + '-limit', [{'fixture_id': 'agent-start', 'repeat': count}],
               'record_count_bound', accepted, 'Framer count only; repeated starts are not a legal lifecycle.')
    for depth, label, accepted in [(32, 'at', True), (33, 'over', False)]:
        recipe('depth-' + label + '-limit', [{'ascii': '{"x":'}, {'ascii': '[', 'repeat': depth - 1},
               {'ascii': 'null'}, {'ascii': ']', 'repeat': depth - 1}, {'ascii': '}\n'}],
               'json_depth_bound', accepted, 'Depth counts root object as1 and each nested object/array as1; field x is intentionally not protocol-admitted.')

    source_manifest = []
    for source_id, (relative, sections) in SOURCES.items():
        path = PACKAGE / relative
        source_manifest.append({'id': source_id, 'path': str(path), 'read_sections': sections,
                                **identity(path.read_bytes())})
    capture = ROOT.parents[2] / 'evidence/implementation/T02/sdk-smoke/metadata-benign.stdout'
    source_manifest.append({'id': 'metadata_capture', 'path': str(capture),
                            'read_sections': ['Complete 2-line stdout supplied by parent; scout did not execute process'],
                            'status': 'parent-retained-runtime-observation', **identity(capture.read_bytes())})
    manifest = {
        'kind': 'hee3.pi-wire-development-fixtures/1', 'profile': 'pi-rpc-earendil-0.85.1',
        'package': '@earendil-works/pi-coding-agent', 'package_version': '0.85.1',
        'status': 'authored-expectations-not-executed-against-parser',
        'authorship': 'Source-backed fixture author separate from initial Rust parser author; not a qualified independent oracle.',
        'fictional_values': 'All authored wire records use synthetic session/request/model/provider/tool/entry IDs, URLs, timestamps, text and usage/cost values. The source manifest separately identifies an actual parent-retained metadata capture; that source is not fictional and was not executed by the fixture author.',
        'module_case_credits': 0, 'provider_requests': 0, 'pi_processes_launched': 0,
        'scope': 'Initial closed worker adapter subset; no compaction, extension, custom entry or tool authority.',
        'worker_policy': {'max_payload_bytes_before_lf': 1048576, 'max_stream_bytes': 8388608,
                          'max_json_depth': 32, 'max_records': 65536,
                          'max_exact_counter': 9007199254740991, 'depth_root_container': 1,
                          'framing': 'Strict UTF-8, LF only, one closed JSON object per frame, unique keys, final LF required; CRLF refused.',
                          'policy_origin': 'Explicit parent-selected Pi worker caps; not inherited HEE3-Control bounds.',
                          'streams': 'Apply 8MiB independently to stdout/stderr. Stderr is diagnostic data only.'},
        'interpretation': [
            'expected_accept is limited to the declared layer, never task or module acceptance.',
            'A record accepted in isolation may still fail lifecycle, correlation, recipe, session attribution or profile enablement.',
            'Retry shapes are admitted parser observations; metadata-only startup disables retry and must separately reject unexpected retry activity.',
            'Known upstream open maps deliberately remain refused until a reviewed closed nested definition is adopted.',
            'Trace stdout paths contain receives only; outbound fixtures in steps specify controller behavior and must not be fed into stdout.',
            'Targeted subsequences are explicit; do not claim they are captured complete real Pi transcripts.',
            'Boundary recipes test only their named layer; materialize by ordered UTF-8/ASCII parts and exact repetition counts, then verify the retained hash.',
            'Exact integers in fixtures use decimal integer tokens; alternate equivalent exponent/decimal lexemes are not decided here.',
        ],
        'sources': source_manifest, 'record_cases': records, 'trace_cases': traces, 'boundary_recipes': recipes,
        'chunking_cases': [
            {'id': 'unicode-one-byte-chunks', 'fixture_id': 'unicode-delta', 'chunk_size': 1,
             'expected_records': 1, 'expected_accept': True,
             'expected_meaning': 'Split inside every UTF-8 code point; retain decoder state and split only LF.'},
            {'id': 'ack-before-final-lf', 'fixture_id': 'prompt-ack', 'split_offsets': [len(ack) - 1],
             'expected_records_before_last_chunk': 0, 'expected_records': 1, 'expected_accept': True,
             'expected_meaning': 'Do not project a record until LF arrives; EOF instead is raw-unterminated fault.'},
        ],
        'startup_scenarios_deferred': ['P29-warning-visibility', 'P30-no-extension', 'P31-ambient-resource'],
        'not_proved': ['Runtime/provider compatibility', 'startup dependency/read/network isolation', 'warning detector qualification',
                       'OS process/descendant/workspace custody', '50-case module qualification', 'collector or module admission'],
    }
    encoded = json.dumps(manifest, ensure_ascii=False, indent=2, allow_nan=False) + '\n'
    (ROOT / 'manifest.json').write_text(encoded, encoding='utf-8')
    print(json.dumps({'records': len(records), 'traces': len(traces), 'boundary_recipes': len(recipes),
                      'manifest': identity(encoded.encode('utf-8'))}, sort_keys=True))


if __name__ == '__main__':
    main()
