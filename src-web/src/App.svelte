<script lang="ts">
  import type { MaybeMusic } from './types_rs'

  import { dialog } from '@tauri-apps/api'
  import { get_maybemusic_by_csv_path } from './invoke'

  let csvMaybeMusic: MaybeMusic[] = []

  function selectCsvFile() {
    dialog
      .open({
        multiple: false,
      })
      .then((csvFilePath) => get_maybemusic_by_csv_path(<string>csvFilePath))
      .then((x) => {
        if (x.ok) {
          csvMaybeMusic = x.object
        } else {
          console.error(x.message)
        }
      })
  }
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

  @media (min-width: 640px) {
    main {
      max-width: none;
    }
  }
</style>
