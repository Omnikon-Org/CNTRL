/**
 * @module components/AuditViewer
 * Audit log viewer showing both AI calls and credential accesses.
 */

import { Component, createSignal, onMount, For, Show } from "solid-js";
import { invoke } from "@tauri-apps/api/core";
import "./AuditViewer.css";

interface AuditEntry {
  id: string;
  entry_type: string;
  intent: string | null;
  tier_used: string | null;
  provider_name: string | null;
  latency_ms: number | null;
  tokens_used: number | null;
  success: boolean | null;
  credential_service: string | null;
  credential_key: string | null;
  access_type: string | null;
  created_at: string;
}

export const AuditViewer: Component = () => {
  const [entries, setEntries] = createSignal<AuditEntry[]>([]);
  const [searchQuery, setSearchQuery] = createSignal("");
  const [filterType, setFilterType] = createSignal<"all" | "ai" | "credential">("all");
  const [currentPage, setCurrentPage] = createSignal(1);
  const [itemsPerPage] = createSignal(10);
  const [isLoading, setIsLoading] = createSignal(false);

  const fetchLog = async () => {
    setIsLoading(true);
    try {
      // Query up to 500 recent entries for client-side search and filtering
      const result = await invoke<AuditEntry[]>("get_recent_audit_log", { limit: 500 });
      setEntries(result);
    } catch (err) {
      console.error("Failed to fetch audit log:", err);
    } finally {
      setIsLoading(false);
    }
  };

  onMount(() => {
    void fetchLog();
  });

  const filteredEntries = () => {
    return entries().filter((entry) => {
      // 1. Filter by type
      if (filterType() === "ai" && entry.entry_type !== "ai_call") return false;
      if (filterType() === "credential" && entry.entry_type !== "credential_access") return false;

      // 2. Filter by search query
      const query = searchQuery().toLowerCase().trim();
      if (!query) return true;

      const intentMatch = entry.intent?.toLowerCase().includes(query) ?? false;
      const providerMatch = entry.provider_name?.toLowerCase().includes(query) ?? false;
      const keyMatch = entry.credential_key?.toLowerCase().includes(query) ?? false;
      const serviceMatch = entry.credential_service?.toLowerCase().includes(query) ?? false;
      const tierMatch = entry.tier_used?.toLowerCase().includes(query) ?? false;
      const typeMatch = entry.access_type?.toLowerCase().includes(query) ?? false;

      return (
        intentMatch ||
        providerMatch ||
        keyMatch ||
        serviceMatch ||
        tierMatch ||
        typeMatch ||
        entry.entry_type.toLowerCase().includes(query)
      );
    });
  };

  const paginatedEntries = () => {
    const start = (currentPage() - 1) * itemsPerPage();
    const end = start + itemsPerPage();
    return filteredEntries().slice(start, end);
  };

  const totalPages = () => {
    return Math.max(1, Math.ceil(filteredEntries().length / itemsPerPage()));
  };

  const handlePrevPage = () => {
    if (currentPage() > 1) {
      setCurrentPage(currentPage() - 1);
    }
  };

  const handleNextPage = () => {
    if (currentPage() < totalPages()) {
      setCurrentPage(currentPage() + 1);
    }
  };

  const formatDate = (isoStr: string) => {
    try {
      const d = new Date(isoStr);
      return d.toLocaleString();
    } catch {
      return isoStr;
    }
  };

  return (
    <div class="audit-viewer">
      <div class="audit-controls">
        <div class="audit-search-wrapper">
          <input
            type="text"
            class="audit-search-input"
            placeholder="Search audit log (e.g. key, intent, provider)..."
            value={searchQuery()}
            onInput={(e) => {
              setSearchQuery(e.currentTarget.value);
              setCurrentPage(1);
            }}
          />
        </div>
        <div class="audit-filters">
          <button
            class={`audit-filter-btn ${filterType() === "all" ? "active" : ""}`}
            onClick={() => {
              setFilterType("all");
              setCurrentPage(1);
            }}
          >
            All Logs
          </button>
          <button
            class={`audit-filter-btn ${filterType() === "ai" ? "active" : ""}`}
            onClick={() => {
              setFilterType("ai");
              setCurrentPage(1);
            }}
          >
            AI Calls
          </button>
          <button
            class={`audit-filter-btn ${filterType() === "credential" ? "active" : ""}`}
            onClick={() => {
              setFilterType("credential");
              setCurrentPage(1);
            }}
          >
            Credentials
          </button>
          <button class="audit-filter-btn" onClick={() => void fetchLog()} disabled={isLoading()}>
            {isLoading() ? "Refreshing..." : "Refresh"}
          </button>
        </div>
      </div>

      <div class="audit-table-wrapper">
        <table class="audit-table">
          <thead>
            <tr>
              <th>Timestamp</th>
              <th>Type</th>
              <th>Details</th>
              <th>Context/Key</th>
              <th>Status/Access</th>
            </tr>
          </thead>
          <tbody>
            <Show
              when={paginatedEntries().length > 0}
              fallback={
                <tr>
                  <td colspan="5" class="audit-empty">
                    {isLoading() ? "Loading audit logs..." : "No matching audit log entries found."}
                  </td>
                </tr>
              }
            >
              <For each={paginatedEntries()}>
                {(entry) => (
                  <tr>
                    <td>{formatDate(entry.created_at)}</td>
                    <td>
                      <span class={`audit-badge ${entry.entry_type === "ai_call" ? "ai" : "key"}`}>
                        {entry.entry_type === "ai_call" ? "AI Call" : "Keychain"}
                      </span>
                    </td>
                    <td>
                      <Show when={entry.entry_type === "ai_call"} fallback={entry.credential_service}>
                        {entry.provider_name} ({entry.tier_used})
                      </Show>
                    </td>
                    <td>
                      <Show when={entry.entry_type === "ai_call"} fallback={entry.credential_key}>
                        <span title={entry.intent || ""}>{entry.intent || "(no prompt)"}</span>
                      </Show>
                    </td>
                    <td>
                      <Show
                        when={entry.entry_type === "ai_call"}
                        fallback={
                          <span class="audit-badge key">{entry.access_type}</span>
                        }
                      >
                        <span class={`audit-badge ${entry.success ? "success" : "failed"}`}>
                          {entry.success ? "OK" : "Error"}
                        </span>
                        <Show when={entry.latency_ms !== null}>
                          <span style={{ "margin-left": "8px", color: "var(--cntrl-fg-muted)" }}>
                            {entry.latency_ms}ms
                          </span>
                        </Show>
                      </Show>
                    </td>
                  </tr>
                )}
              </For>
            </Show>
          </tbody>
        </table>
      </div>

      <div class="audit-pagination">
        <span class="audit-pagination-info">
          Showing page {currentPage()} of {totalPages()} ({filteredEntries().length} total entries)
        </span>
        <div class="audit-pagination-btns">
          <button class="audit-page-btn" onClick={handlePrevPage} disabled={currentPage() === 1}>
            Previous
          </button>
          <button class="audit-page-btn" onClick={handleNextPage} disabled={currentPage() === totalPages()}>
            Next
          </button>
        </div>
      </div>
    </div>
  );
};
