export interface ProtocolNode {
  id: string;
  remark: string;
  children?: ProtocolNode[];
}

export function containsId(nodes: ProtocolNode[], id: string): boolean {
  return nodes.some(
    (node) => node.id === id || (node.children ? containsId(node.children, id) : false)
  );
}

export function insertNode(
  nodes: ProtocolNode[],
  parentId: string | null,
  newNode: ProtocolNode
): ProtocolNode[] {
  if (parentId === null) {
    return [...nodes, newNode];
  }

  return nodes.map((node) => {
    if (node.id === parentId) {
      return { ...node, children: [...(node.children ?? []), newNode] };
    }
    if (!node.children) {
      return node;
    }
    return { ...node, children: insertNode(node.children, parentId, newNode) };
  });
}

export function removeNode(nodes: ProtocolNode[], id: string): ProtocolNode[] {
  return nodes
    .filter((node) => node.id !== id)
    .map((node) =>
      node.children
        ? { ...node, children: removeNode(node.children, id) }
        : node
    );
}

export function findNode(
  nodes: ProtocolNode[],
  id: string
): ProtocolNode | undefined {
  for (const node of nodes) {
    if (node.id === id) return node;
    if (node.children) {
      const found = findNode(node.children, id);
      if (found) return found;
    }
  }
}

export function updateNode(
  nodes: ProtocolNode[],
  id: string,
  update: (node: ProtocolNode) => ProtocolNode
): ProtocolNode[] {
  return nodes.map((node) => {
    if (node.id === id) {
      return update(node);
    }
    if (!node.children) {
      return node;
    }
    return { ...node, children: updateNode(node.children, id, update) };
  });
}
