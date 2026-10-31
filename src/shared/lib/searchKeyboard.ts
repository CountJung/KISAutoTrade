interface SearchEnterEvent {
  key: string
  isComposing?: boolean
  keyCode?: number
  nativeEvent?: { isComposing?: boolean; keyCode?: number }
}

// Only key/composition detection. Input target, validity and callbacks belong to each caller.
export function isSearchEnter(event: SearchEnterEvent): boolean {
  return event.key === 'Enter'
    && !event.isComposing && !event.nativeEvent?.isComposing
    && event.keyCode !== 229 && event.nativeEvent?.keyCode !== 229
}
