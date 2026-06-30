export function autoScroll(node: HTMLElement, isActive: boolean) {
  if (isActive) {
    node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
  }
  return {
    update(newIsActive: boolean) {
      if (newIsActive) {
        node.scrollIntoView({ behavior: 'smooth', block: 'nearest' });
      }
    }
  };
}
