import { invoke } from '@tauri-apps/api'

import type { FeResult } from './types'
import type { MaybeMusic } from './types_rs'

export function get_maybemusic_by_csv_path(
  csvPath: string
): Promise<FeResult<MaybeMusic[]>> {
  return invoke('get_maybemusic_by_csv_path', {
    csvPath,
  })
}
