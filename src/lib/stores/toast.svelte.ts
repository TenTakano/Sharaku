type ToastType = "success" | "error";

interface Toast {
  id: number;
  type: ToastType;
  message: string;
}

let nextId = 0;
let toasts = $state<Toast[]>([]);

export function addToast(
  type: ToastType,
  message: string,
  duration: number = 3000,
) {
  const id = nextId++;
  toasts.push({ id, type, message });
  setTimeout(() => removeToast(id), duration);
}

export function removeToast(id: number) {
  toasts = toasts.filter((t) => t.id !== id);
}

export function getToasts(): Toast[] {
  return toasts;
}
