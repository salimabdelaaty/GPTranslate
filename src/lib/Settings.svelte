<script lang="ts">
    import { invoke } from "@tauri-apps/api/core";
    import { onMount } from "svelte";
    import AppIcon from "./AppIcon.svelte";
    import pkg from "../../package.json";
    const version = pkg.version;
    let config = $state({
        api_provider: "openai",
        openai_api_key: "",
        azure_endpoint: "",
        azure_api_key: "",
        azure_api_version: "2025-01-01-preview",
        azure_deployment_name: "gpt-4.1-nano",
        model: "gpt-4.1-nano",
        target_language: "English",
        alternative_target_language: "Norwegian",
        auto_start: true,
        hotkey: "Ctrl+Alt+C",
        theme: "auto",
        minimize_to_tray: true,
        custom_prompt: "",
    });
    let isValidatingApiKey = $state(false);
    let apiKeyValid = $state<boolean | null>(null);
    let isSaving = $state(false);
    let saveMessage = $state("");
    let azureEndpointInfo = $state<{
        isValid: boolean;
        type?: string;
        deploymentDetected?: string;
        apiVersionDetected?: string;
    } | null>(null);

    interface Props {
        onClose: () => void;
    }
    let { onClose }: Props = $props();
    onMount(async () => {
        console.log("Settings component mounted");
        try {
            config = (await invoke("get_config")) as any;
        } catch (e) {
            console.error("Failed to load config:", e);
        }
    });

    async function onApiProviderChange() {
        // Set default model values based on provider
        if (config.api_provider === "openai" && !config.model) {
            config.model = "gpt-4.1-nano";
        } else if (
            config.api_provider === "azure_openai" &&
            !config.azure_deployment_name
        ) {
            config.azure_deployment_name = "gpt-4.1-nano";
        }
        apiKeyValid = null;
    }
    function parseAzureEndpoint(url: string): {
        baseUrl: string;
        apiVersion: string;
        deploymentName: string | null;
        isModelsEndpoint: boolean;
        isValid: boolean;
        errorMessage?: string;
    } {
        try {
            const parsedUrl = new URL(url);

            // Extract API version from query parameters
            const apiVersion = parsedUrl.searchParams.get("api-version");

            // Determine endpoint type based on hostname
            const isModelsEndpoint = parsedUrl.hostname.includes(
                "services.ai.azure.com",
            );
            const isCognitiveServicesEndpoint = parsedUrl.hostname.includes(
                "cognitiveservices.azure.com",
            );

            if (!isModelsEndpoint && !isCognitiveServicesEndpoint) {
                return {
                    baseUrl: url,
                    apiVersion: config.azure_api_version,
                    deploymentName: null,
                    isModelsEndpoint: false,
                    isValid: false,
                    errorMessage:
                        "URL must be either a cognitiveservices.azure.com or services.ai.azure.com endpoint",
                };
            }

            // Extract deployment name for cognitive services endpoints
            let deploymentName: string | null = null;
            if (isCognitiveServicesEndpoint) {
                const pathParts = parsedUrl.pathname
                    .split("/")
                    .filter((part) => part.length > 0);
                const deploymentIndex = pathParts.findIndex(
                    (part) => part === "deployments",
                );

                if (
                    deploymentIndex !== -1 &&
                    deploymentIndex + 1 < pathParts.length
                ) {
                    deploymentName = pathParts[deploymentIndex + 1];
                } else {
                    // If no deployment in path, we can still extract from a full endpoint URL
                    console.log(
                        "No deployment found in path, cognitive services endpoint may need deployment name",
                    );
                }
            }

            // Create base URL (without path and query parameters)
            const baseUrl = `${parsedUrl.protocol}//${parsedUrl.host}`;

            return {
                baseUrl,
                apiVersion: apiVersion || config.azure_api_version,
                deploymentName,
                isModelsEndpoint,
                isValid: true,
            };
        } catch (e) {
            console.error("Error parsing Azure endpoint URL:", e);
            return {
                baseUrl: url,
                apiVersion: config.azure_api_version,
                deploymentName: null,
                isModelsEndpoint: false,
                isValid: false,
                errorMessage: `Invalid URL format: ${e instanceof Error ? e.message : "Unknown error"}`,
            };
        }
    }
    function onAzureEndpointChange() {
        azureEndpointInfo = null; // Reset info

        if (config.azure_endpoint) {
            try {
                const {
                    baseUrl,
                    apiVersion,
                    deploymentName,
                    isModelsEndpoint,
                    isValid,
                    errorMessage,
                } = parseAzureEndpoint(config.azure_endpoint);

                if (!isValid) {
                    console.warn(`Invalid Azure endpoint: ${errorMessage}`);
                    azureEndpointInfo = { isValid: false };
                    return;
                }

                // Update the config with parsed values
                config.azure_endpoint = baseUrl;

                if (apiVersion) {
                    config.azure_api_version = apiVersion;
                    console.log(`✓ Extracted API version: ${apiVersion}`);
                }

                if (isModelsEndpoint) {
                    // For models endpoints, clear deployment name as it's not needed
                    config.azure_deployment_name = "";
                    console.log(
                        "✓ Models API endpoint detected - deployment name cleared",
                    );
                } else if (deploymentName) {
                    // For cognitive services endpoints, use extracted deployment name
                    config.azure_deployment_name = deploymentName;
                    console.log(
                        `✓ Extracted deployment name: ${deploymentName}`,
                    );
                } else {
                    // Cognitive services endpoint but no deployment name in URL
                    console.log(
                        "⚠ Cognitive Services endpoint detected but no deployment name found in URL. You may need to specify it manually.",
                    );
                }

                // Set endpoint info for UI feedback
                azureEndpointInfo = {
                    isValid: true,
                    type: isModelsEndpoint
                        ? "Models API"
                        : "Cognitive Services",
                    deploymentDetected: deploymentName || undefined,
                    apiVersionDetected: apiVersion || undefined,
                };

                console.log(
                    `✓ Endpoint type: ${isModelsEndpoint ? "Models API" : "Cognitive Services"}`,
                );
                console.log(`✓ Base URL set to: ${baseUrl}`);
            } catch (e) {
                console.error("Failed to parse Azure endpoint:", e);
                azureEndpointInfo = { isValid: false };
            }
        }
        // Reset API key validation when endpoint changes
        apiKeyValid = null;
    }

    async function validateApiKey() {
        if (
            (config.api_provider === "openai" && !config.openai_api_key) ||
            (config.api_provider === "azure_openai" &&
                (!config.azure_api_key || !config.azure_endpoint))
        ) {
            return;
        }

        isValidatingApiKey = true;
        try {
            const isValid = (await invoke("validate_api_key", {
                apiProvider: config.api_provider,
                apiKey:
                    config.api_provider === "openai"
                        ? config.openai_api_key
                        : config.azure_api_key,
                endpoint:
                    config.api_provider === "azure_openai"
                        ? config.azure_endpoint
                        : null,
                apiVersion:
                    config.api_provider === "azure_openai"
                        ? config.azure_api_version
                        : null,
            })) as boolean;
            apiKeyValid = isValid;
        } catch (e) {
            console.error("API key validation failed:", e);
            apiKeyValid = false;
        } finally {
            isValidatingApiKey = false;
        }
    }

    async function saveSettings() {
        isSaving = true;
        saveMessage = "";
        try {
            await invoke("save_config", { newConfig: config });
            saveMessage =
                "Settings saved successfully! Hotkey changes take effect immediately.";
            setTimeout(() => {
                saveMessage = "";
            }, 5000);
        } catch (e) {
            console.error("Failed to save settings:", e);
            saveMessage = "Failed to save settings: " + e;
        } finally {
            isSaving = false;
        }
    }
    function resetToDefaults() {
        if (
            confirm("Are you sure you want to reset all settings to defaults?")
        ) {
            config = {
                api_provider: "openai",
                openai_api_key: "",
                azure_endpoint: "",
                azure_api_key: "",
                azure_api_version: "2025-01-01-preview",
                azure_deployment_name: "gpt-4.1-nano",
                model: "gpt-4.1-nano",
                target_language: "English",
                alternative_target_language: "Norwegian",
                auto_start: true,
                hotkey: "CommandOrControl+Alt+C",
                theme: "auto",
                minimize_to_tray: true,
                custom_prompt:
                    "Translate the given text from {detected_language} to {target_language} accurately while preserving the meaning, tone, and nuance of the original content.\n\n# Additional Details\n- Ensure the translation retains the context, cultural meaning, tone, formal/informal style, and any idiomatic expressions.\n- Do **not** alter names, technical terms, or specific formatting unless required for grammatical correctness in the target language.\n- If the detected language is the same as the target language, choose the most appropriate alternative language for translation.\n\n# Output Format\nThe translation output should be provided as valid JSON containing 'detected_language' and 'translated_text' fields.\n\n# Notes\n- Ensure punctuation and capitalization match the norms of the target language.\n- When encountering idiomatic expressions, adapt them to equivalent phrases in the target language rather than direct word-for-word translation.\n- For ambiguous content, aim for the most contextually appropriate meaning.\n- Take into consideration the whole text and what it is about.",
            };
            apiKeyValid = null;
        }
    }
</script>

<div class="settings-overlay">
    <div class="settings-container">
        <div class="settings-header">
            <div class="settings-header-content">
                <AppIcon size={48} className="settings-logo" />
                <h1 class="settings-title">GPTranslate</h1>
            </div>
            <button
                class="close-btn"
                onclick={onClose}
                title="Close settings"
                aria-label="Close settings"
            >
                <i class="bi bi-x-lg"></i>
            </button>
        </div>

        <div class="settings-content">
            <!-- API Configuration -->
            <section class="settings-section">
                <h3><i class="bi bi-cloud"></i>API Configuration</h3>

                <div class="form-group">
                    <label for="api-provider">API Provider</label>
                    <select
                        id="api-provider"
                        bind:value={config.api_provider}
                        onchange={onApiProviderChange}
                    >
                        <option value="openai">OpenAI</option>
                        <option value="azure_openai">Azure OpenAI</option>
                    </select>
                </div>

                {#if config.api_provider === "openai"}
                    <div class="form-group">
                        <label for="openai-key">OpenAI API Key</label>
                        <div class="api-key-group">
                            <input
                                id="openai-key"
                                type="password"
                                bind:value={config.openai_api_key}
                                placeholder="sk-..."
                                onblur={validateApiKey}
                            />
                            {#if isValidatingApiKey}
                                <span class="validation-icon validating">
                                    <i class="bi bi-arrow-clockwise"></i>
                                </span>
                            {:else if apiKeyValid === true}
                                <span class="validation-icon valid">
                                    <i class="bi bi-check-circle-fill"></i>
                                </span>
                            {:else if apiKeyValid === false}
                                <span class="validation-icon invalid">
                                    <i class="bi bi-x-circle-fill"></i>
                                </span>
                            {/if}
                        </div>
                    </div>
                {:else}
                    <div class="form-group">
                        <label for="azure-endpoint">Azure OpenAI Endpoint</label
                        >
                        <input
                            id="azure-endpoint"
                            type="url"
                            bind:value={config.azure_endpoint}
                            placeholder="Paste your full Azure OpenAI endpoint URL here..."
                            onblur={validateApiKey}
                            oninput={onAzureEndpointChange}
                        />
                        <small>
                            Paste the complete endpoint URL from Azure portal.
                            Supported formats:
                            <br />
                            •
                            <code
                                >https://resource.cognitiveservices.azure.com/openai/...</code
                            >
                            <br />
                            •
                            <code
                                >https://resource.services.ai.azure.com/models/...</code
                            >
                            <br />
                            The app will automatically extract the base URL, API
                            version, and deployment name.
                        </small>

                        {#if azureEndpointInfo?.isValid}
                            <div class="endpoint-info success">
                                <i class="bi bi-check-circle-fill"></i>
                                <strong>Auto-detected:</strong>
                                {azureEndpointInfo.type} endpoint
                                {#if azureEndpointInfo.deploymentDetected}
                                    • Deployment: <code
                                        >{azureEndpointInfo.deploymentDetected}</code
                                    >
                                {/if}
                                {#if azureEndpointInfo.apiVersionDetected}
                                    • API Version: <code
                                        >{azureEndpointInfo.apiVersionDetected}</code
                                    >
                                {/if}
                            </div>
                        {:else if azureEndpointInfo?.isValid === false}
                            <div class="endpoint-info error">
                                <i class="bi bi-exclamation-triangle-fill"></i>
                                <strong>Invalid endpoint format.</strong> Please
                                use a valid Azure OpenAI endpoint URL.
                            </div>
                        {/if}
                    </div>

                    <div class="form-group">
                        <label for="azure-key">Azure API Key</label>
                        <div class="api-key-group">
                            <input
                                id="azure-key"
                                type="password"
                                bind:value={config.azure_api_key}
                                placeholder="Your Azure API key"
                                onblur={validateApiKey}
                            />
                            {#if isValidatingApiKey}
                                <span class="validation-icon validating">
                                    <i class="bi bi-arrow-clockwise"></i>
                                </span>
                            {:else if apiKeyValid === true}
                                <span class="validation-icon valid">
                                    <i class="bi bi-check-circle-fill"></i>
                                </span>
                            {:else if apiKeyValid === false}
                                <span class="validation-icon invalid">
                                    <i class="bi bi-x-circle-fill"></i>
                                </span>
                            {/if}
                        </div>
                    </div>
                    <div class="form-group">
                        <label for="azure-deployment"
                            >Azure Deployment Name</label
                        >
                        <input
                            id="azure-deployment"
                            type="text"
                            bind:value={config.azure_deployment_name}
                            placeholder="gpt-4"
                        />
                    </div>

                    <div class="form-group">
                        <label for="azure-api-version">Azure API Version</label>
                        <input
                            id="azure-api-version"
                            type="text"
                            bind:value={config.azure_api_version}
                            placeholder="2025-01-01-preview"
                        />
                        <small>
                            API version for Azure OpenAI requests (e.g.,
                            2025-01-01-preview)
                        </small>
                    </div>
                {/if}

                {#if config.api_provider === "openai"}
                    <div class="form-group">
                        <label for="model">OpenAI Model</label>
                        <input
                            id="model"
                            type="text"
                            bind:value={config.model}
                            placeholder="gpt-4.1-nano"
                        />
                        <small>
                            Specify the OpenAI model to use (e.g., gpt-4.1-nano,
                            gpt-4o-mini, gpt-4, etc.)
                        </small>
                    </div>
                {/if}
            </section>
            <!-- App Behavior -->
            <section class="settings-section">
                <h3><i class="bi bi-sliders"></i>App Behavior</h3>

                <div class="form-group">
                    <label for="target-language">Target Language</label>
                    <input
                        id="target-language"
                        type="text"
                        bind:value={config.target_language}
                        placeholder="English, Spanish, French, etc."
                    />
                    <small
                        >Specify the default language to translate to. This
                        language will be used in the custom prompt as
                        &#123;target_language&#125;.</small
                    >
                </div>

                <div class="form-group">
                    <label for="alternative-target-language"
                        >Alternative Target Language</label
                    >
                    <input
                        id="alternative-target-language"
                        type="text"
                        bind:value={config.alternative_target_language}
                        placeholder="Norwegian, Spanish, German, etc."
                    />
                    <small
                        >Language to use when the detected language is the same
                        as the target language. For example, if you normally
                        translate to English, but the input is already English,
                        it will translate to this alternative language instead.</small
                    >
                </div>

                <div class="form-group">
                    <label for="theme">Theme</label>
                    <select id="theme" bind:value={config.theme}>
                        <option value="auto">Auto (System)</option>
                        <option value="light">Light</option>
                        <option value="dark">Dark</option>
                    </select>
                </div>
                <div class="form-group">
                    <label for="hotkey">Global Hotkey</label>
                    <input
                        id="hotkey"
                        type="text"
                        bind:value={config.hotkey}
                        placeholder="CommandOrControl+Alt+C"
                    />
                    <small
                        >Example: CommandOrControl+Alt+C, Alt+Shift+T, etc.</small
                    >
                </div>

                <div class="checkbox-group">
                    <label class="checkbox-label">
                        <input
                            type="checkbox"
                            bind:checked={config.auto_start}
                        />
                        <span class="checkmark"></span>
                        Start with Windows
                    </label>
                </div>
                <div class="checkbox-group">
                    <label class="checkbox-label">
                        <input
                            type="checkbox"
                            bind:checked={config.minimize_to_tray}
                        />
                        <span class="checkmark"></span>
                        Minimize to system tray
                    </label>
                </div>
            </section>
            <!-- Custom Prompt -->
            <section class="settings-section">
                <h3>
                    <i class="bi bi-chat-text"></i>Custom Translation Prompt
                </h3>

                <div class="form-group">
                    <label for="custom-prompt">Translation Instructions</label>
                    <textarea
                        id="custom-prompt"
                        bind:value={config.custom_prompt}
                        placeholder="Enter custom instructions for the AI translator..."
                        class="prompt-textarea"
                        rows="8"
                    ></textarea>
                    <small>
                        Customize how the AI translates text. You can use these
                        variables in your prompt:
                        <br />
                        <code>&#123;detected_language&#125;</code> - The
                        automatically detected source language
                        <br />
                        <code>&#123;target_language&#125;</code> - Your configured
                        target language
                    </small>
                </div>
            </section>

            <!-- About -->
            <section class="settings-section">
                <h3><i class="bi bi-info-circle"></i>About</h3>

                <div class="about-content">
                    <div class="about-item">
                        <strong>Developer:</strong> Phil Berndt
                    </div>
                    <div class="about-item">
                        <strong>Email:</strong> phil@berndt.no
                    </div>
                    <div class="about-item">
                        <strong>Website:</strong>
                        <a
                            href="https://berndt.no"
                            target="_blank"
                            rel="noopener noreferrer"
                        >
                            https://berndt.no
                        </a>
                    </div>
                    <div class="about-item">
                        <strong>Version:</strong>
                        {version}
                    </div>
                </div>
            </section>
        </div>

        <div class="settings-footer">
            {#if saveMessage}
                <div
                    class="save-message"
                    class:success={saveMessage.includes("successfully")}
                    class:error={!saveMessage.includes("successfully")}
                >
                    {saveMessage}
                </div>
            {/if}
            <div class="settings-actions">
                <button class="reset-btn" onclick={resetToDefaults}>
                    <i class="bi bi-arrow-counterclockwise"></i>Reset to
                    Defaults
                </button>
                <button
                    class="save-btn"
                    onclick={saveSettings}
                    disabled={isSaving}
                >
                    {#if isSaving}
                        <i class="bi bi-arrow-clockwise spinning"></i>Saving...
                    {:else}
                        <i class="bi bi-check-lg"></i>Save Settings
                    {/if}
                </button>
            </div>
        </div>
    </div>
</div>

<style>
    * {
        box-sizing: border-box;
    }

    .settings-overlay {
        position: fixed;
        top: 0;
        left: 0;
        right: 0;
        bottom: 0;
        background: rgba(0, 0, 0, 0.5);
        display: flex;
        align-items: center;
        justify-content: center;
        z-index: 1000;
        padding: 20px;
    }
    .settings-container {
        background: white;
        border-radius: 12px;
        width: 100%;
        max-width: 600px;
        max-height: 90vh;
        display: flex;
        flex-direction: column;
        box-shadow: 0 10px 30px rgba(0, 0, 0, 0.3);
        overflow: hidden;
    }
    .settings-header {
        display: flex;
        justify-content: space-between;
        align-items: center;
        padding: 24px 24px 20px 24px;
        border-bottom: 1px solid #e0e0e0;
        flex-shrink: 0;
    }

    .settings-header-content {
        display: flex;
        align-items: center;
        justify-content: center;
        flex: 1;
        gap: 12px;
    }

    .settings-title {
        margin: 0;
        font-size: 1.8rem;
        font-weight: 600;
        color: #333;
        letter-spacing: -0.02em;
    }

    :global(.settings-logo) {
        margin-right: 0 !important;
        filter: drop-shadow(0 2px 4px rgba(0, 0, 0, 0.1));
    }

    .close-btn {
        background: none;
        border: none;
        font-size: 1.2rem;
        color: #666;
        cursor: pointer;
        padding: 8px;
        border-radius: 6px;
        transition: all 0.2s;
    }

    .close-btn:hover {
        background: #f0f0f0;
        color: #333;
    }
    .settings-content {
        flex: 1;
        overflow-y: auto;
        overflow-x: hidden;
        padding: 0 24px;
        box-sizing: border-box;
    }
    .settings-section {
        margin: 24px 0;
        min-width: 0; /* Prevent flex item from overflowing */
    }

    .settings-section h3 {
        font-size: 1.1rem;
        color: #333;
        margin: 0 0 16px 0;
        display: flex;
        align-items: center;
        gap: 8px;
        padding-bottom: 8px;
        border-bottom: 1px solid #e0e0e0;
    }
    .form-group {
        margin-bottom: 16px;
        min-width: 0; /* Prevent flex item from overflowing */
    }

    .form-group label {
        display: block;
        font-weight: 500;
        margin-bottom: 4px;
        color: #333;
    }
    .form-group input,
    .form-group select {
        width: 100%;
        max-width: 100%;
        padding: 10px 12px;
        border: 1px solid #ddd;
        border-radius: 6px;
        font-family: inherit;
        font-size: 14px;
        transition: border-color 0.2s;
        box-sizing: border-box;
        min-width: 0; /* Prevent input from overflowing */
    }
    .form-group input:focus,
    .form-group select:focus {
        outline: none;
        border-color: #379df1;
        box-shadow: 0 0 0 2px rgba(55, 157, 241, 0.1);
    }
    .form-group small {
        color: #666;
        font-size: 12px;
        margin-top: 4px;
        display: block;
    }

    .form-group small code {
        background: #f1f3f4;
        color: #d73a49;
        padding: 2px 4px;
        border-radius: 3px;
        font-family: "Consolas", "Monaco", "Courier New", monospace;
        font-size: 11px;
    }
    .api-key-group {
        position: relative;
        display: flex;
        align-items: center;
        width: 100%;
        max-width: 100%;
        box-sizing: border-box;
    }
    .api-key-group input {
        padding-right: 40px;
        flex: 1;
        min-width: 0; /* Prevent input from overflowing */
    }

    .validation-icon {
        position: absolute;
        right: 12px;
        font-size: 16px;
    }
    .validation-icon.validating {
        color: #379df1;
        animation: spin 1s linear infinite;
    }

    .validation-icon.valid {
        color: #28a745;
    }
    .validation-icon.invalid {
        color: #dc3545;
    }

    .endpoint-info {
        margin-top: 8px;
        padding: 8px 12px;
        border-radius: 6px;
        font-size: 13px;
        display: flex;
        align-items: flex-start;
        gap: 8px;
        border: 1px solid;
    }

    .endpoint-info.success {
        background: #d4edda;
        color: #155724;
        border-color: #c3e6cb;
    }

    .endpoint-info.error {
        background: #f8d7da;
        color: #721c24;
        border-color: #f5c6cb;
    }

    .endpoint-info i {
        font-size: 14px;
        margin-top: 1px;
        flex-shrink: 0;
    }

    .endpoint-info code {
        background: rgba(0, 0, 0, 0.1);
        padding: 2px 4px;
        border-radius: 3px;
        font-family: "Consolas", "Monaco", "Courier New", monospace;
        font-size: 11px;
    }

    .checkbox-group {
        margin-bottom: 12px;
    }

    .checkbox-label {
        display: flex;
        align-items: center;
        cursor: pointer;
        font-weight: normal;
        user-select: none;
    }

    .checkbox-label input[type="checkbox"] {
        width: auto;
        margin-right: 8px;
    }

    .settings-footer {
        padding: 20px 24px;
        border-top: 1px solid #e0e0e0;
        flex-shrink: 0;
    }

    .save-message {
        padding: 8px 12px;
        border-radius: 6px;
        margin-bottom: 12px;
        font-size: 14px;
    }

    .save-message.success {
        background: #d4edda;
        color: #155724;
        border: 1px solid #c3e6cb;
    }

    .save-message.error {
        background: #f8d7da;
        color: #721c24;
        border: 1px solid #f5c6cb;
    }

    .settings-actions {
        display: flex;
        gap: 12px;
        justify-content: flex-end;
    }

    .reset-btn,
    .save-btn {
        padding: 10px 20px;
        border: none;
        border-radius: 6px;
        font-family: inherit;
        font-size: 14px;
        font-weight: 500;
        cursor: pointer;
        transition: all 0.2s;
        display: flex;
        align-items: center;
        gap: 6px;
    }

    .reset-btn {
        background: #6c757d;
        color: white;
    }

    .reset-btn:hover {
        background: #5a6268;
    }
    .save-btn {
        background: #379df1;
        color: white;
    }

    .save-btn:hover:not(:disabled) {
        background: #2980e6;
    }

    .save-btn:disabled {
        opacity: 0.6;
        cursor: not-allowed;
    }

    .spinning {
        animation: spin 1s linear infinite;
    }
    @keyframes spin {
        from {
            transform: rotate(0deg);
        }
        to {
            transform: rotate(360deg);
        }
    }

    .prompt-textarea {
        width: 100%;
        max-width: 100%;
        padding: 12px;
        border: 1px solid #ddd;
        border-radius: 6px;
        font-family: "Consolas", "Monaco", "Courier New", monospace;
        font-size: 13px;
        line-height: 1.4;
        resize: vertical;
        min-height: 120px;
        transition: border-color 0.2s;
        box-sizing: border-box;
    }

    .prompt-textarea:focus {
        outline: none;
        border-color: #379df1;
        box-shadow: 0 0 0 2px rgba(55, 157, 241, 0.1);
    }

    .about-content {
        background: #f8f9fa;
        border: 1px solid #e0e0e0;
        border-radius: 8px;
        padding: 16px;
    }

    .about-item {
        margin-bottom: 8px;
        font-size: 14px;
        line-height: 1.5;
    }

    .about-item:last-child {
        margin-bottom: 0;
    }

    .about-item strong {
        color: #333;
        min-width: 80px;
        display: inline-block;
    }
    .about-item a {
        color: #379df1;
        text-decoration: none;
        transition: color 0.2s;
    }

    .about-item a:hover {
        color: #2980e6;
        text-decoration: underline;
    }

    @media (prefers-color-scheme: dark) {
        .settings-container {
            background: #2d2d2d;
            color: #f6f6f6;
        }

        .settings-header {
            border-color: #444;
        }

        .settings-title {
            color: #f6f6f6;
        }

        .close-btn {
            color: #ccc;
        }

        .close-btn:hover {
            background: #444;
            color: #f6f6f6;
        }

        .settings-section h3 {
            color: #f6f6f6;
            border-color: #444;
        }

        .form-group label {
            color: #f6f6f6;
        }

        .form-group input,
        .form-group select {
            background: #3a3a3a;
            border-color: #555;
            color: #f6f6f6;
        }
        .form-group input:focus,
        .form-group select:focus {
            border-color: #379df1;
            background: #404040;
        }

        .form-group small {
            color: #ccc;
        }

        .form-group small code {
            background: #444;
            color: #7dd3fc;
        }

        .settings-footer {
            border-color: #444;
        }

        .prompt-textarea {
            background: #404040;
            border-color: #444;
            color: #f6f6f6;
        }
        .prompt-textarea:focus {
            border-color: #379df1;
            background: #4a4a4a;
        }

        .about-content {
            background: #383838;
            border-color: #444;
        }

        .about-item strong {
            color: #f6f6f6;
        }
        .about-item a {
            color: #379df1;
        }

        .about-item a:hover {
            color: #2980e6;
        }

        .save-message.success {
            background: #1e4620;
            color: #a3cfac;
            border-color: #2d5930;
        }
        .save-message.error {
            background: #4a1e23;
            color: #f5c6cb;
            border-color: #5c2329;
        }

        .endpoint-info.success {
            background: #1e4620;
            color: #a3cfac;
            border-color: #2d5930;
        }

        .endpoint-info.error {
            background: #4a1e23;
            color: #f5c6cb;
            border-color: #5c2329;
        }

        .endpoint-info code {
            background: rgba(255, 255, 255, 0.1);
            color: #7dd3fc;
        }
    }

    @media (max-width: 768px) {
        .settings-overlay {
            padding: 10px;
        }

        .settings-container {
            max-height: 95vh;
        }

        .settings-actions {
            flex-direction: column;
        }

        .reset-btn,
        .save-btn {
            width: 100%;
            justify-content: center;
        }
    }
</style>
