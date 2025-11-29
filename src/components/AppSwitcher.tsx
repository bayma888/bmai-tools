import type { AppId } from "@/lib/api";
import { ProviderIcon } from "@/components/ProviderIcon";
import { cn } from "@/lib/utils";

interface AppSwitcherProps {
  activeApp: AppId;
  onSwitch: (app: AppId) => void;
}

const apps: { id: AppId; icon: string; name: string }[] = [
  { id: "claude", icon: "claude", name: "Claude" },
  { id: "codex", icon: "openai", name: "Codex" },
  { id: "gemini", icon: "gemini", name: "Gemini" },
];

export function AppSwitcher({ activeApp, onSwitch }: AppSwitcherProps) {
  return (
    <div className="flex flex-col gap-1">
      {apps.map((app) => {
        const isActive = activeApp === app.id;
        return (
          <button
            key={app.id}
            type="button"
            onClick={() => onSwitch(app.id)}
            className={cn(
              "flex items-center gap-2.5 px-3 py-2 rounded-lg text-sm font-medium transition-all duration-200",
              isActive
                ? "bg-[hsl(var(--sidebar-accent))] text-[hsl(var(--sidebar-foreground))]"
                : "text-[hsl(var(--sidebar-muted))] hover:bg-[hsl(var(--sidebar-accent))] hover:text-[hsl(var(--sidebar-foreground))]"
            )}
          >
            <ProviderIcon
              icon={app.icon}
              name={app.name}
              size={18}
              className={cn(
                "transition-colors",
                isActive
                  ? "text-[hsl(var(--sidebar-foreground))]"
                  : "text-[hsl(var(--sidebar-muted))]"
              )}
            />
            <span>{app.name}</span>
            {isActive && (
              <span className="ml-auto w-1.5 h-1.5 rounded-full bg-primary" />
            )}
          </button>
        );
      })}
    </div>
  );
}
