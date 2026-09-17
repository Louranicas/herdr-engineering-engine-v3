#!/usr/bin/env node
// Credential-free metadata-only Pi 0.85.1 SDK entry for the T02 development smoke.

const warningFault = process.argv.includes("--warning-fault");
if (process.env.PI_OFFLINE !== "1") throw new Error("PI_OFFLINE=1 is required");
const originalEmitWarning = process.emitWarning;

const pi = await import("file:///pi/dist/index.js");
const piAi = await import("file:///pi/node_modules/@earendil-works/pi-ai/dist/index.js");
const setupSignal = AbortSignal.timeout(10_000);
const modelRuntime = await pi.ModelRuntime.create({
  credentials: new piAi.InMemoryCredentialStore(),
  modelsStore: new piAi.InMemoryModelsStore(),
  modelsPath: null,
  allowModelNetwork: false,
  refreshOnCreate: false,
  signal: setupSignal,
});

const cwd = "/work";
const agentDir = "/agent";
const createRuntime = async ({ sessionManager, sessionStartEvent }) => {
  const settingsManager = pi.SettingsManager.inMemory(
    { retry: { enabled: false }, compaction: { enabled: false } },
    { projectTrusted: false },
  );
  const services = await pi.createAgentSessionServices({
    cwd,
    agentDir,
    settingsManager,
    modelRuntime,
    modelRuntimeSignal: setupSignal,
    resourceLoaderOptions: {
      extensionFactories: [],
      additionalExtensionPaths: [],
      additionalSkillPaths: [],
      additionalPromptTemplatePaths: [],
      additionalThemePaths: [],
      noExtensions: true,
      noSkills: true,
      noPromptTemplates: true,
      noThemes: true,
      noContextFiles: true,
      systemPrompt: "HEE3 credential-free metadata-only probe.",
      appendSystemPrompt: [],
    },
  });
  const created = await pi.createAgentSessionFromServices({
    services,
    sessionManager,
    sessionStartEvent,
    tools: [],
    noTools: "all",
    customTools: [],
    thinkingLevel: "off",
  });
  const loader = services.resourceLoader;
  const resourceCount =
    loader.getExtensions().extensions.length + loader.getSkills().skills.length +
    loader.getPrompts().prompts.length + loader.getThemes().themes.length +
    loader.getAgentsFiles().agentsFiles.length + loader.getAppendSystemPrompt().length;
  const errors = [...settingsManager.drainErrors(), ...services.diagnostics];
  const credentials = await modelRuntime.listCredentials({ signal: setupSignal });
  if (errors.length || resourceCount || created.session.getActiveToolNames().length || credentials.length || modelRuntime.getAvailableSnapshot().length) {
    throw new Error("isolated startup readback differed");
  }
  return { ...created, services, diagnostics: services.diagnostics };
};

const runtime = await pi.createAgentSessionRuntime(createRuntime, {
  cwd,
  agentDir,
  sessionManager: pi.SessionManager.inMemory(cwd),
});
if (process.emitWarning !== originalEmitWarning) throw new Error("warning emitter was replaced during SDK startup");
if (warningFault) process.emitWarning("HEE3_T02_VISIBLE_WARNING_CONTROL");
await pi.runRpcMode(runtime);
