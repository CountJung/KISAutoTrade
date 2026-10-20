/** Remove only the immutable draft that was submitted; preserve newer edits. */
export function removeDraftIfUnchanged<T>(map: Record<string, T>, id: string, submittedDraft: T | undefined): Record<string, T> {
  if (!Object.is(map[id], submittedDraft)) return map
  const next = { ...map }
  delete next[id]
  return next
}
