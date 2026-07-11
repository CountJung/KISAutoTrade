import { invoke } from './transport'
import type {
  DatabaseConfigView,
  DatabaseStatusView,
  DatabaseTransferResult,
  JsonStorageInventoryView,
  SaveDatabaseConfigInput,
  StorageBackend,
} from './databaseTypes'

export const getDatabaseConfig = (): Promise<DatabaseConfigView> =>
  invoke('get_database_config')

export const saveDatabaseConfig = (input: SaveDatabaseConfigInput): Promise<DatabaseConfigView> =>
  invoke('save_database_config', { input })

export const testDatabaseConnection = (): Promise<DatabaseStatusView> =>
  invoke('test_database_connection')

export const createDatabaseTables = (): Promise<DatabaseStatusView> =>
  invoke('create_database_tables')

export const clearDatabaseTables = (confirmation: string): Promise<DatabaseStatusView> =>
  invoke('clear_database_tables', { confirmation })

export const dropDatabaseTables = (confirmation: string): Promise<DatabaseStatusView> =>
  invoke('drop_database_tables', { confirmation })

export const inspectJsonStorage = (): Promise<JsonStorageInventoryView> =>
  invoke('inspect_json_storage')

export const importJsonToDatabase = (): Promise<DatabaseTransferResult> =>
  invoke('import_json_to_database')

export const exportDatabaseToJson = (): Promise<DatabaseTransferResult> =>
  invoke('export_database_to_json')

export const setStorageBackend = (backend: StorageBackend): Promise<DatabaseStatusView> =>
  invoke('set_storage_backend', { backend })
