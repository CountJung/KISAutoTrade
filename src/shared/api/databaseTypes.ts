export type DatabaseProvider = 'postgresql' | 'mariadb'
export type DatabaseTlsMode = 'disable' | 'prefer' | 'require'
export type StorageBackend = 'json' | 'database'

export interface DatabaseConfigView {
  provider: DatabaseProvider
  host: string
  port: number
  database: string
  username: string
  passwordConfigured: boolean
  tlsMode: DatabaseTlsMode
  maxConnections: number
  activeBackend: StorageBackend
  configured: boolean
}

export interface SaveDatabaseConfigInput {
  provider: DatabaseProvider
  host: string
  port: number
  database: string
  username: string
  /** 비어 있거나 생략하면 기존 저장 password를 유지합니다. */
  password?: string
  tlsMode: DatabaseTlsMode
  maxConnections: number
}

export interface DatabaseTableStatus {
  name: string
  purpose: string
  exists: boolean
  rowCount: number
}

export interface DatabaseStatusView {
  connected: boolean
  provider: DatabaseProvider
  activeBackend: StorageBackend
  schemaVersion: number | null
  requiredSchemaVersion: number
  serverVersion: string | null
  latencyMs: number | null
  checkedAt: string
  message: string
  tables: DatabaseTableStatus[]
}

export interface JsonStorageCategoryView {
  category: string
  fileCount: number
  sizeBytes: number
}

export interface JsonStorageInventoryView {
  fileCount: number
  sizeBytes: number
  categories: JsonStorageCategoryView[]
}

export interface DatabaseTransferResult {
  operation: 'jsonToDatabase' | 'databaseToJson'
  processed: number
  insertedOrUpdated: number
  skipped: number
  sizeBytes: number
  outputPath: string | null
  checksum: string
  completedAt: string
  message: string
}
