import { useMemo } from "react";
import { GripVertical, Check } from "lucide-react";
import { useTranslation } from "react-i18next";
import type {
  DraggableAttributes,
  DraggableSyntheticListeners,
} from "@dnd-kit/core";
import type { Provider } from "@/types";
import type { AppId } from "@/lib/api";
import { cn } from "@/lib/utils";
import { ProviderActions } from "@/components/providers/ProviderActions";
import { ProviderIcon } from "@/components/ProviderIcon";
import UsageFooter from "@/components/UsageFooter";

interface DragHandleProps {
  attributes: DraggableAttributes;
  listeners: DraggableSyntheticListeners;
  isDragging: boolean;
}

interface ProviderCardProps {
  provider: Provider;
  isCurrent: boolean;
  appId: AppId;
  onSwitch: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onConfigureUsage: (provider: Provider) => void;
  onOpenWebsite: (url: string) => void;
  onDuplicate: (provider: Provider) => void;
  dragHandleProps?: DragHandleProps;
}

const extractApiUrl = (provider: Provider, fallbackText: string) => {
  if (provider.notes?.trim()) {
    return provider.notes.trim();
  }

  if (provider.websiteUrl) {
    return provider.websiteUrl;
  }

  const config = provider.settingsConfig;

  if (config && typeof config === "object") {
    const envBase =
      (config as Record<string, any>)?.env?.ANTHROPIC_BASE_URL ||
      (config as Record<string, any>)?.env?.GOOGLE_GEMINI_BASE_URL;
    if (typeof envBase === "string" && envBase.trim()) {
      return envBase;
    }

    const baseUrl = (config as Record<string, any>)?.config;

    if (typeof baseUrl === "string" && baseUrl.includes("base_url")) {
      const match = baseUrl.match(/base_url\s*=\s*['"]([^'"]+)['"]/);
      if (match?.[1]) {
        return match[1];
      }
    }
  }

  return fallbackText;
};

export function ProviderCard({
  provider,
  isCurrent,
  appId,
  onSwitch,
  onEdit,
  onDelete,
  onConfigureUsage,
  onOpenWebsite,
  onDuplicate,
  dragHandleProps,
}: ProviderCardProps) {
  const { t } = useTranslation();

  const fallbackUrlText = t("provider.notConfigured", {
    defaultValue: "未配置接口地址",
  });

  const displayUrl = useMemo(() => {
    return extractApiUrl(provider, fallbackUrlText);
  }, [provider, fallbackUrlText]);

  const isClickableUrl = useMemo(() => {
    if (provider.notes?.trim()) {
      return false;
    }
    if (displayUrl === fallbackUrlText) {
      return false;
    }
    return true;
  }, [provider.notes, displayUrl, fallbackUrlText]);

  const usageEnabled = provider.meta?.usage_script?.enabled ?? false;

  const handleOpenWebsite = () => {
    if (!isClickableUrl) {
      return;
    }
    onOpenWebsite(displayUrl);
  };

  return (
    <div
      className={cn(
        "group relative bg-card rounded-xl border transition-all duration-200",
        isCurrent
          ? "border-primary shadow-sm"
          : "border-border hover:border-primary/30 hover:shadow-sm",
        dragHandleProps?.isDragging && "shadow-lg scale-[1.02] z-10"
      )}
    >
      <div className="p-4">
        <div className="flex items-start gap-3">
          {/* 拖拽手柄 */}
          <button
            type="button"
            className={cn(
              "flex-shrink-0 cursor-grab active:cursor-grabbing p-1 -ml-1 mt-0.5",
              "text-muted-foreground/40 hover:text-muted-foreground transition-colors",
              "opacity-0 group-hover:opacity-100",
              dragHandleProps?.isDragging && "cursor-grabbing opacity-100"
            )}
            aria-label={t("provider.dragHandle")}
            {...(dragHandleProps?.attributes ?? {})}
            {...(dragHandleProps?.listeners ?? {})}
          >
            <GripVertical className="h-4 w-4" />
          </button>

          {/* 供应商图标 */}
          <div
            className={cn(
              "flex-shrink-0 h-10 w-10 rounded-lg flex items-center justify-center",
              "bg-muted/50 border border-border"
            )}
          >
            <ProviderIcon
              icon={provider.icon}
              name={provider.name}
              color={provider.iconColor}
              size={22}
            />
          </div>

          {/* 供应商信息 */}
          <div className="flex-1 min-w-0">
            <div className="flex items-center gap-2">
              <h3 className="text-sm font-semibold text-foreground truncate">
                {provider.name}
              </h3>
              {provider.category === "third_party" &&
                provider.meta?.isPartner && (
                  <span
                    className="text-yellow-500 dark:text-yellow-400 text-xs"
                    title={t("provider.officialPartner", {
                      defaultValue: "官方合作伙伴",
                    })}
                  >
                    ⭐
                  </span>
                )}
              {isCurrent && (
                <span className="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-xs font-medium bg-primary/10 text-primary">
                  <Check className="h-3 w-3" />
                  {t("provider.currentlyUsing")}
                </span>
              )}
            </div>

            {displayUrl && (
              <button
                type="button"
                onClick={handleOpenWebsite}
                className={cn(
                  "mt-1 text-xs max-w-full truncate block",
                  isClickableUrl
                    ? "text-primary/80 hover:text-primary hover:underline cursor-pointer"
                    : "text-muted-foreground cursor-default"
                )}
                title={displayUrl}
                disabled={!isClickableUrl}
              >
                {displayUrl}
              </button>
            )}
          </div>

          {/* 用量信息和操作按钮 */}
          <div className="flex items-center gap-2 flex-shrink-0">
            <div className="transition-all duration-200 group-hover:opacity-0 group-hover:w-0 group-hover:overflow-hidden">
              <UsageFooter
                provider={provider}
                providerId={provider.id}
                appId={appId}
                usageEnabled={usageEnabled}
                isCurrent={isCurrent}
                inline={true}
              />
            </div>

            <div className="flex items-center gap-1 opacity-0 group-hover:opacity-100 transition-opacity duration-200">
              <ProviderActions
                isCurrent={isCurrent}
                onSwitch={() => onSwitch(provider)}
                onEdit={() => onEdit(provider)}
                onDuplicate={() => onDuplicate(provider)}
                onConfigureUsage={() => onConfigureUsage(provider)}
                onDelete={() => onDelete(provider)}
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
