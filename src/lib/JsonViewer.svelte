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

    const sav_file = listen<{json: any}>("sav_file", (result) => {
        if (typeof result.payload === "string") {
            content.set({
                json: JSON.parse(result.payload)
            });
            console.log(content);
        }

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
    });
</script>

<div class="viewer">
    <div class="json">
        <JSONEditor bind:content={$content}/>
    </div>
    <div class="actions">
        <button on:click={() => invoke("save_file", {content: JSON.stringify($content.json)})}>Save</button>
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
        background-color: #f0f0f0;
    }
</style>