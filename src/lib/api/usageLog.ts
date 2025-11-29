import { invoke } from "@tauri-apps/api/core";

export interface UsageLogData {
  name?: string;
  id: string;
  isActive: boolean;
  createdAt: string;
  expiresAt?: string;
  description?: string;
  usage?: {
    total?: {
      requests?: number;
      inputTokens?: number;
      outputTokens?: number;
      cacheCreateTokens?: number;
      cacheReadTokens?: number;
      allTokens?: number;
      cost?: number;
      formattedCost?: string;
    };
  };
  limits?: {
    concurrencyLimit?: number;
    dailyCostLimit?: number;
    totalCostLimit?: number;
    currentDailyCost?: number;
    currentTotalCost?: number;
    weeklyOpusCost?: number;
  };
  accounts?: {
    claudeAccountId?: string;
    geminiAccountId?: string;
    openaiAccountId?: string;
  };
}

export interface UsageLogResult {
  success: boolean;
  data?: UsageLogData;
  error?: string;
}

// 模型统计数据
export interface ModelStatsItem {
  model: string;
  requests: number;
  inputTokens: number;
  outputTokens: number;
  cacheCreateTokens: number;
  cacheReadTokens: number;
  allTokens: number;
  costs: {
    input: number;
    output: number;
    cacheWrite: number;
    cacheRead: number;
    total: number;
  };
  formatted: {
    input: string;
    output: string;
    cacheWrite: string;
    cacheRead: string;
    total: string;
  };
  pricing: {
    input: number;
    output: number;
    cacheWrite: number;
    cacheRead: number;
  };
}

export interface ModelStatsResult {
  success: boolean;
  data?: ModelStatsItem[];
  period?: string;
  error?: string;
}

export const usageLogApi = {
  // 查询用户基本用量信息
  async query(
    apiKey: string,
    baseUrl?: string,
    period?: string,
  ): Promise<UsageLogResult> {
    try {
      const result = await invoke<UsageLogResult>("query_api_usage", {
        apiKey,
        baseUrl: baseUrl || null,
        period: period || "daily",
      });
      return result;
    } catch (error: unknown) {
      return {
        success: false,
        error: typeof error === "string" ? error : String(error),
      };
    }
  },

  // 查询模型统计数据
  async queryModelStats(
    apiKey: string,
    baseUrl?: string,
    period?: string,
  ): Promise<ModelStatsResult> {
    try {
      const result = await invoke<ModelStatsResult>("query_model_stats", {
        apiKey,
        baseUrl: baseUrl || null,
        period: period || "daily",
      });
      return result;
    } catch (error: unknown) {
      return {
        success: false,
        error: typeof error === "string" ? error : String(error),
      };
    }
  },
};
