<script lang="ts">
    import {JSONEditor} from 'svelte-jsoneditor'
    import {writable} from 'svelte/store';
    import {listen} from "@tauri-apps/api/event";
    import {invoke} from "@tauri-apps/api/core";
    import {onDestroy} from "svelte";

    let content = writable<{json: any}>({
        json: {
            "empty": "drag a player.sav file on to edit it",
        }
    });

    let save_type = writable<number>(0);

    let path = writable<string>("");

    const sav_file = listen<string>("sav_file", (result) => {
        content.set({
            json: JSON.parse(result.payload[0])
        });
        save_type.set(Number(result.payload[1]));
    });

    const sav_path = listen<string>("sav_path", (result) => {
        path.set(result.payload);
    });

    const open_err = listen("open_err", (result) => {
        if (typeof result.payload === "string") {
            content.set({
                json: {
                    err: result.payload,
                }
            });
            console.log(content);
        }
    });

    onDestroy(() => {
        sav_file.then((f) => f());
        open_err.then((f) => f());
        sav_path.then((f) => f());
    });
</script>

<div class="viewer">
    <div class="json">
        <JSONEditor bind:content={$content}/>
    </div>
    <div class="actions">
        <div class="left-items">
        <button on:click={() => invoke("save_file", {
            json: JSON.stringify($content.json),
            save_type: $save_type,
            path: $path,
        })}>Save</button>
        Always make a backup before saving - File Type {$save_type}
        </div>
        <div class="right-items">
            <button class="right-button" on:click={() => invoke("open_saves_folder")}>Open Saves Folder</button>
        </div>
    </div>

</div>

<style>
    .json {
        flex: 1;
        overflow: auto;
    }
    .viewer {
        display: flex;
        flex-direction: column;
        height: 100vh;
        width: 100%;
    }
    .actions {
        display: flex;
        justify-content: space-between;
        background-color: #f0f0f0;
    }
</style>