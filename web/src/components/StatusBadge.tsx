import type { DeliveryStatus } from "../lib/escrow";

const LABELS: Record<DeliveryStatus, string> = {
  Created: "Awaiting pickup",
  PickedUp: "In transit",
  Completed: "Delivered ✓",
  Refunded: "Refunded",
};

const COLORS: Record<DeliveryStatus, string> = {
  Created: "#b45309",
  PickedUp: "#1d4ed8",
  Completed: "#15803d",
  Refunded: "#6b7280",
};

interface Props {
  status: DeliveryStatus;
}

export function StatusBadge({ status }: Props) {
  return (
    <span
      className="status-badge"
      style={{ backgroundColor: COLORS[status] }}
      aria-label={`Delivery status: ${LABELS[status]}`}
    >
      {LABELS[status]}
    </span>
  );
}
