import { CSS } from "@dnd-kit/utilities";
import { DndContext, closestCenter } from "@dnd-kit/core";
import {
  SortableContext,
  useSortable,
  verticalListSortingStrategy,
} from "@dnd-kit/sortable";
import type { CSSProperties } from "react";
import type { Provider } from "@/types";
import type { AppId } from "@/lib/api";
import { useDragSort } from "@/hooks/useDragSort";
import { ProviderCard } from "@/components/providers/ProviderCard";
import { ProviderEmptyState } from "@/components/providers/ProviderEmptyState";

interface ProviderListProps {
  providers: Record<string, Provider>;
  currentProviderId: string;
  appId: AppId;
  onSwitch: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onDuplicate: (provider: Provider) => void;
  onConfigureUsage?: (provider: Provider) => void;
  onOpenWebsite: (url: string) => void;
  onCreate?: () => void;
  isLoading?: boolean;
}

export function ProviderList({
  providers,
  currentProviderId,
  appId,
  onSwitch,
  onEdit,
  onDelete,
  onDuplicate,
  onConfigureUsage,
  onOpenWebsite,
  onCreate,
  isLoading = false,
}: ProviderListProps) {
  const { sortedProviders, sensors, handleDragEnd } = useDragSort(
    providers,
    appId,
  );

  if (isLoading) {
    return (
      <div className="space-y-3">
        {[0, 1, 2].map((index) => (
          <div
            key={index}
            className="h-20 w-full rounded-xl border border-border bg-muted/30 animate-pulse"
          />
        ))}
      </div>
    );
  }

  if (sortedProviders.length === 0) {
    return <ProviderEmptyState onCreate={onCreate} />;
  }

  return (
    <DndContext
      sensors={sensors}
      collisionDetection={closestCenter}
      onDragEnd={handleDragEnd}
    >
      <SortableContext
        items={sortedProviders.map((provider) => provider.id)}
        strategy={verticalListSortingStrategy}
      >
        <div className="space-y-3">
          {sortedProviders.map((provider, index) => (
            <div
              key={provider.id}
              className="animate-slide-up"
              style={{ animationDelay: `${index * 50}ms` }}
            >
              <SortableProviderCard
                provider={provider}
                isCurrent={provider.id === currentProviderId}
                appId={appId}
                onSwitch={onSwitch}
                onEdit={onEdit}
                onDelete={onDelete}
                onDuplicate={onDuplicate}
                onConfigureUsage={onConfigureUsage}
                onOpenWebsite={onOpenWebsite}
              />
            </div>
          ))}
        </div>
      </SortableContext>
    </DndContext>
  );
}

interface SortableProviderCardProps {
  provider: Provider;
  isCurrent: boolean;
  appId: AppId;
  onSwitch: (provider: Provider) => void;
  onEdit: (provider: Provider) => void;
  onDelete: (provider: Provider) => void;
  onDuplicate: (provider: Provider) => void;
  onConfigureUsage?: (provider: Provider) => void;
  onOpenWebsite: (url: string) => void;
}

function SortableProviderCard({
  provider,
  isCurrent,
  appId,
  onSwitch,
  onEdit,
  onDelete,
  onDuplicate,
  onConfigureUsage,
  onOpenWebsite,
}: SortableProviderCardProps) {
  const {
    setNodeRef,
    attributes,
    listeners,
    transform,
    transition,
    isDragging,
  } = useSortable({ id: provider.id });

  const style: CSSProperties = {
    transform: CSS.Transform.toString(transform),
    transition,
  };

  return (
    <div ref={setNodeRef} style={style}>
      <ProviderCard
        provider={provider}
        isCurrent={isCurrent}
        appId={appId}
        onSwitch={onSwitch}
        onEdit={onEdit}
        onDelete={onDelete}
        onDuplicate={onDuplicate}
        onConfigureUsage={
          onConfigureUsage ? (item) => onConfigureUsage(item) : () => undefined
        }
        onOpenWebsite={onOpenWebsite}
        dragHandleProps={{
          attributes,
          listeners,
          isDragging,
        }}
      />
    </div>
  );
}
