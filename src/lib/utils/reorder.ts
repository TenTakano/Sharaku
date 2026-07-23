export function moveItem<T>(items: T[], fromItem: T, toItem: T): T[] {
  const from = items.indexOf(fromItem);
  const to = items.indexOf(toItem);
  if (from === -1 || to === -1 || from === to) return items;

  const result = [...items];
  const [moved] = result.splice(from, 1);
  result.splice(to, 0, moved);
  return result;
}
