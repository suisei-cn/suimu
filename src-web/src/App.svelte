<script lang="ts">
    import { invoke, dialog } from '@tauri-apps/api'

    let csvFilePath = "";

    function selectCsvFile() {
        dialog.open({
            multiple: false
        }).then(x => {
            csvFilePath = <string> x
            console.log(`CSV file path: ${csvFilePath}`)
        })
    }
</script>

<main>
    <h1>Suisei-music Companion</h1>
    <hr>
    <div>
        <button on:click={selectCsvFile}>
            Select CSV file
        </button>
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