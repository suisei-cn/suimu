<script lang="ts">
  import type { MaybeMusic } from './types_rs'

  import { dialog } from '@tauri-apps/api'
  import { get_maybemusic_by_csv_path } from './invoke'

  import dayjs from 'dayjs'
  import { Grid } from 'gridjs'
  import { onMount } from 'svelte'

  let csvMaybeMusic: MaybeMusic[] = []
  let grid: Grid = undefined

  let musicTable

  function selectCsvFile() {
    dialog
      .open({
        multiple: false,
      })
      .then((csvFilePath) => get_maybemusic_by_csv_path(<string>csvFilePath))
      .then((x) => {
        if (x.ok) {
          csvMaybeMusic = x.object
          updateTable(x.object)
        } else {
          console.error(x.message)
        }
      })
  }

  const isNope = (v: any) => v === undefined || v === null

  function toClipRange(fr: number | null, to: number | null): string {
    if (isNope(fr) && isNope(to)) {
      return '(full)'
    }
    if (isNope(fr)) {
      return `(begin) - ${to.toFixed(2)}`
    }
    if (isNope(to)) {
      return `${fr.toFixed(2)} - (end)`
    }
    return `${fr.toFixed(2)} - ${to.toFixed(2)}`
  }

  function updateTable(data: MaybeMusic[]) {
    grid
      .updateConfig({
        data: data.map((r) => [
          dayjs(r.datetime).format('YYYY-MM-DD HH:mm'),
          r.video_type,
          r.video_id,
          toClipRange(r.clip_start, r.clip_end),
          r.title,
          r.artist,
          r.performer,
          r.comment,
        ]),
      })
      .forceRender()
  }

  onMount(() => {
    grid = new Grid({
      columns: [
        'Date',
        'Video Type',
        'Video ID',
        'Range',
        'Title',
        'Artist',
        'Performer',
        'Comment',
      ],
      data: [],
      fixedHeader: true,
      height: '50vh',
    }).render(musicTable)
  })
</script>

<main>
  <h1>Suisei-music Companion</h1>
  <hr />
  <div>
    <button on:click="{selectCsvFile}"> Select CSV file </button>
    {#if csvMaybeMusic.length}
      <div>
        {csvMaybeMusic.length} clips found.
      </div>
    {/if}
    <div id="musicTableDiv" bind:this="{musicTable}"></div>
  </div>
</main>

<style>
  main {
    text-align: center;
    padding: 1em;
    max-width: 240px;
    margin: 0 auto;
  }

  h1 {
    color: #ff3e00;
    text-transform: uppercase;
    font-size: 4em;
    font-weight: 100;
  }

  #musicTableDiv {
    margin-top: 15px;
  }

  @media (min-width: 640px) {
    main {
      max-width: none;
    }
  }
</style>
