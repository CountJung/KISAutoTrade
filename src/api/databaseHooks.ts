import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query'

import * as cmd from './commands'
import { KEYS } from './queryKeys'
import type {
  DatabaseStatusView,
  DatabaseTransferResult,
  SaveDatabaseConfigInput,
  StorageBackend,
} from './types'

export function useDatabaseConfig(enabled = true) {
  return useQuery({
    queryKey: KEYS.databaseConfig,
    queryFn: cmd.getDatabaseConfig,
    staleTime: Infinity,
    enabled,
  })
}

export function useJsonStorageInventory(enabled = true) {
  return useQuery({
    queryKey: KEYS.jsonStorageInventory,
    queryFn: cmd.inspectJsonStorage,
    staleTime: 30_000,
    enabled,
  })
}

function useStatusMutation<T>(mutationFn: (input: T) => Promise<DatabaseStatusView>) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: (status) => {
      queryClient.setQueryData(KEYS.databaseStatus, status)
      void queryClient.invalidateQueries({ queryKey: KEYS.databaseConfig })
    },
  })
}

export function useSaveDatabaseConfig() {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn: (input: SaveDatabaseConfigInput) => cmd.saveDatabaseConfig(input),
    onSuccess: (config) => {
      queryClient.setQueryData(KEYS.databaseConfig, config)
      queryClient.removeQueries({ queryKey: KEYS.databaseStatus })
    },
  })
}

export function useTestDatabaseConnection() {
  return useStatusMutation<void>(() => cmd.testDatabaseConnection())
}

export function useCreateDatabaseTables() {
  return useStatusMutation<void>(() => cmd.createDatabaseTables())
}

export function useClearDatabaseTables() {
  return useStatusMutation<string>((confirmation) => cmd.clearDatabaseTables(confirmation))
}

export function useDropDatabaseTables() {
  return useStatusMutation<string>((confirmation) => cmd.dropDatabaseTables(confirmation))
}

export function useSetStorageBackend() {
  return useStatusMutation<StorageBackend>((backend) => cmd.setStorageBackend(backend))
}

function useTransferMutation(mutationFn: () => Promise<DatabaseTransferResult>) {
  const queryClient = useQueryClient()
  return useMutation({
    mutationFn,
    onSuccess: () => {
      void queryClient.invalidateQueries({ queryKey: KEYS.databaseStatus })
      void queryClient.invalidateQueries({ queryKey: KEYS.jsonStorageInventory })
    },
  })
}

export function useImportJsonToDatabase() {
  return useTransferMutation(cmd.importJsonToDatabase)
}

export function useExportDatabaseToJson() {
  return useTransferMutation(cmd.exportDatabaseToJson)
}
