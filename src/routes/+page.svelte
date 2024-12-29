<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { stat } from '@tauri-apps/plugin-fs';
	import { onMount } from 'svelte';
	import {
		type Image,
		type Parameters,
		type Success,
		Status,
		type ProcessError
	} from '../types/types';
	import { open } from '@tauri-apps/plugin-dialog';
	import { basename, extname, dirname } from '@tauri-apps/api/path';
	import { listen } from '@tauri-apps/api/event';
	import { fade, slide } from 'svelte/transition';
	import { convertFileSrc } from '@tauri-apps/api/core';
	import { getVersion } from '@tauri-apps/api/app';
	import { getCurrentWebview } from '@tauri-apps/api/webview';
	let quality = $state(80);
	let resize = $state('NoResizing');
	let resizeTo = $state(1600);
	let compression = $state('lossless');
	let isAllowEnlarging = $state(false);
	let saveTo = $state('same-folder');
	let saveFolder = $state('');
	let images = $state<Image[]>([]);
	let done = $state(0);
	let inProgress = $state(false);
	let showAbout = $state(false);
	let version = $state('');
	let dropInProgress = $state(false);

	onMount(() => {
		const setupDragDrop = async () => {
			const unlisten = await getCurrentWebview().onDragDropEvent((event) => {
				if (!showAbout && !inProgress) {
					if (event.payload.type === 'over') {
						dropInProgress = true;
					} else if (event.payload.type === 'drop') {
						let files = event.payload.paths;
						if (files != null) {
							console.log('User dropped', files);
							addFiles(files);
						}
						dropInProgress = false;
					} else {
						console.log('File drop cancelled');
						dropInProgress = false;
					}
				}
			});

			return unlisten;
		};

		let unlisten: (() => void) | undefined;

		setupDragDrop()
			.then((fn) => {
				unlisten = fn;
			})
			.catch((error) => {
				console.error('Error setting up drag-and-drop:', error);
			});

		return () => {
			if (unlisten) {
				unlisten();
			}
		};
	});

	async function extractFileDetails(filePath: string) {
		let filepath = '';
		let filename = '';
		let extension = '';
		try {
			filepath = await dirname(filePath);
			filename = await basename(filePath, await extname(filePath)); // Basename without extension
			extension = await extname(filePath);
		} catch (error) {
			console.error('Error extracting file details:', error);
		}

		return { filepath, filename, extension };
	}

	async function getFileSize(filePath: string) {
		const fileMetadata = await stat(filePath);
		let fileSizeKB = 0;
		if (fileMetadata.size !== undefined) {
			fileSizeKB = Math.round(fileMetadata.size / 1024);
		} else {
			fileSizeKB = -1;
		}
		return fileSizeKB;
	}

	async function processImages(event: Event) {
		event.preventDefault();
		images.forEach((image) => {
			image.inProgress = false;
			image.status = Status.TODO;
		});
		let parameters: Parameters = {
			isLossless: compression === 'lossless',
			quality: quality,
			resize: resize,
			resizeTo: resizeTo,
			isEnlargingAllowed: isAllowEnlarging,
			saveFolder: saveFolder
		};
		done = 0;
		inProgress = true;
		await invoke('process', { images, parameters });
	}

	async function cancel(event: Event) {
		event.preventDefault();
		done = 0;
		inProgress = false;
		clearListProgress();
		await invoke('cancel_process', {});
	}

	async function getFolder() {
		const folder = await open({
			multiple: false,
			directory: true
		});
		if (folder != null) {
			saveFolder = folder;
		}
	}

	async function addFilesAction() {
		const files = await open({
			multiple: true,
			directory: false,
			filters: [
				{
					name: 'Images',
					extensions: ['png', 'jpeg', 'jpg']
				}
			]
		});
		if (files != null) {
			addFiles(files);
		}
	}

	async function addFiles(files: string[]) {
		files.forEach(async (file) => {
			let { filepath, filename, extension } = await extractFileDetails(file);
			let newImage: Image = {
				fullPath: file,
				filename: filename,
				extension: extension,
				path: filepath,
				originalSize: await getFileSize(file),
				webpSize: 0,
				status: Status.TODO,
				errorMessage: '',
				inProgress: false
			};
			images.push(newImage);
		});
	}

	function clear() {
		images = [];
	}

	function clearListProgress() {
		images.forEach((image) => {
			image.inProgress = false;
		});
	}

	function updateListProgress(imagePath: string) {
		images.forEach((image) => {
			if (image.fullPath == imagePath) {
				image.inProgress = true;
			}
		});
	}

	function updateListSuccess(success: Success) {
		images.forEach((image) => {
			if (image.fullPath == success.fullPath) {
				image.status = Status.SUCCESS;
				image.webpSize = success.size;
				image.inProgress = false;
			}
		});
	}

	function updateListError(error: ProcessError) {
		images.forEach((image) => {
			if (image.fullPath == error.fullPath) {
				image.status = Status.ERROR;
				image.errorMessage = error.error;
				image.inProgress = false;
			}
		});
	}

	function checkProgress() {
		done++;
		if (done == images.length) {
			inProgress = false;
		}
	}

	async function toggleAbout() {
		showAbout = !showAbout;
		if (version === '') {
			version = await getVersion();
		}
	}

	listen<string>('progress', (event) => {
		console.log(`Progress started for ${event.payload}`);
		updateListProgress(event.payload);
	});

	listen<Success>('success', (event) => {
		console.log(`Succes for ${event.payload.fullPath} with size ${event.payload.size}`);
		updateListSuccess(event.payload);
		checkProgress();
	});

	listen<ProcessError>('error', (event) => {
		console.log(`Error for ${event.payload.fullPath} with ${event.payload.error}`);
		updateListError(event.payload);
		checkProgress();
	});

	export function dragover(ev: DragEvent) {
		ev.preventDefault();
		if (ev.dataTransfer != null) {
			ev.dataTransfer.dropEffect = 'move';
		}
	}

	function dropFile(ev: DragEvent) {
		ev.preventDefault();
		console.log('Drop: ' + ev);
		if (ev.dataTransfer != null) {
			const files = ev.dataTransfer.files;
			console.log('Files: ' + files);
		}
	}
</script>

<main class="flex">
	{#if !showAbout && !dropInProgress}
		<section class="flex-grow min-w-96 h-screen flex flex-col">
			<!-- svelte-ignore a11y_no_static_element_interactions -->
			<div
				class="flex-grow w-auto h-auto px-2 flex flex-col overflow-y-auto my-2"
				class:justify-center={images.length == 0}
			>
				{#if images.length == 0}
					<div class="text-center text-slate-500">
						<div class="text-2xl mb-6">Drop your images here!</div>
						<div class="text-2xl mb-6">Or use the Add Images button below.</div>
						<div class="">Accepted formats: PNG, JPEG, WEBP.</div>
					</div>
				{/if}
				{#each images as image, i}
					<div
						class="flex justify-between border-b-2 border-gray-800 py-3 first:pt-0"
						in:slide={{ duration: 50, delay: 50 * i }}
					>
						<div class="flex flex-row justify-start">
							<div
								class="w-12 h-12 flex items-center justify-center overflow-hidden rounded border border-zinc-950"
							>
								<img
									src={convertFileSrc(image.fullPath)}
									alt={image.filename}
									class="w-full h-full object-cover"
								/>
							</div>
							<div class="ms-3">
								<div class="py-1 first:pt-0 text-md truncate max-w-fit">
									{image.filename + image.extension}
								</div>
								<div class="truncate">{image.path}</div>
							</div>
						</div>
						<div class="me-2 min-w-12">
							{#if image.inProgress}
								<div class="h-6">
									<svg
										width="60"
										height="15"
										viewBox="0 0 120 30"
										xmlns="http://www.w3.org/2000/svg"
										fill="#477A91"
									>
										<circle cx="15" cy="15" r="15">
											<animate
												attributeName="r"
												from="15"
												to="15"
												begin="0s"
												dur="0.8s"
												values="15;9;15"
												calcMode="linear"
												repeatCount="indefinite"
											/>
											<animate
												attributeName="fill-opacity"
												from="1"
												to="1"
												begin="0s"
												dur="0.8s"
												values="1;.5;1"
												calcMode="linear"
												repeatCount="indefinite"
											/>
										</circle>
										<circle cx="60" cy="15" r="9" fill-opacity="0.3">
											<animate
												attributeName="r"
												from="9"
												to="9"
												begin="0s"
												dur="0.8s"
												values="9;15;9"
												calcMode="linear"
												repeatCount="indefinite"
											/>
											<animate
												attributeName="fill-opacity"
												from="0.5"
												to="0.5"
												begin="0s"
												dur="0.8s"
												values=".5;1;.5"
												calcMode="linear"
												repeatCount="indefinite"
											/>
										</circle>
										<circle cx="105" cy="15" r="15">
											<animate
												attributeName="r"
												from="15"
												to="15"
												begin="0s"
												dur="0.8s"
												values="15;9;15"
												calcMode="linear"
												repeatCount="indefinite"
											/>
											<animate
												attributeName="fill-opacity"
												from="1"
												to="1"
												begin="0s"
												dur="0.8s"
												values="1;.5;1"
												calcMode="linear"
												repeatCount="indefinite"
											/>
										</circle>
									</svg>
								</div>
							{/if}
							{#if !image.inProgress}
								<div
									class="text-right text-md py-1 first:pt-0 truncate"
									class:text-success={image.status == Status.SUCCESS}
									class:text-primary={image.status == Status.TODO}
									class:text-error={image.status == Status.ERROR}
								>
									{image.status}{image.status == Status.ERROR && image.errorMessage !== ''
										? ' : ' + image.errorMessage
										: ''}
								</div>
							{/if}
							<div class="text-right truncate">
								{image.originalSize} KB {image.status == Status.SUCCESS
									? '> ' + image.webpSize + ' KB'
									: ''}
							</div>
						</div>
					</div>
				{/each}
			</div>

			<div class="flex justify-between py-2 me-2">
				{#if !inProgress}
					<button
						class="btn btn-primary btn-sm mx-2"
						onclick={addFilesAction}
						in:fade={{ duration: 50 }}>Add images</button
					>
					<div class="flex flex-row">
						<button
							class="btn btn-neutral btn-sm mx-2"
							onclick={toggleAbout}
							in:fade={{ duration: 50 }}>About...</button
						>
						<button class="btn btn-neutral btn-sm" onclick={clear} in:fade={{ duration: 50 }}
							>Clear images</button
						>
					</div>
				{/if}
				{#if inProgress}
					<progress
						class="progress progress-secondary w-full mx-2 h-8"
						value={done}
						max={images.length}
						in:fade={{ duration: 50 }}
					></progress>
					<!-- <progress class="progress progress-primary w-full mx-2 h-8" value="37" max="172"></progress> -->
				{/if}
			</div>
		</section>
		<section class="min-w-60 w-60 h-screen flex flex-col">
			<div class="h-auto flex-grow px-2 overflow-y-auto">
				<div class="border-b-2 border-gray-800 pb-3">
					<div class="py-2">Image compression</div>
					<div class="">
						<div class="form-control">
							<label class="label cursor-pointer justify-start">
								<input
									type="radio"
									name="compression"
									class="radio radio-primary"
									value="lossless"
									bind:group={compression}
								/>
								<span class="ms-2">Lossless</span>
							</label>
						</div>
						<div class="form-control">
							<label class="label cursor-pointer justify-start">
								<input
									type="radio"
									name="compression"
									class="radio radio-primary"
									value="lossy"
									bind:group={compression}
								/>
								<span class="ms-2">Lossy</span>
							</label>
						</div>
					</div>
					<label class="flex items-center gap-2">
						Quality
						<input
							type="number"
							class="input input-bordered input-primary input-sm w-16"
							placeholder="80"
							bind:value={quality}
							disabled={compression !== 'lossy'}
						/>
					</label>
					<div class="flex items-center space-x-2 pt-2">
						<input
							type="range"
							min="0"
							max="100"
							class="range range-primary disabled-range"
							bind:value={quality}
							disabled={compression !== 'lossy'}
							class:disabled-range={compression !== 'lossy'}
						/>
					</div>
				</div>
				<div class="border-b-2 border-gray-800 pb-3">
					<div class="py-2">Image resize</div>
					<div class="">
						<div class="form-control">
							<label class="label cursor-pointer justify-start">
								<input
									type="radio"
									name="resize"
									class="radio radio-primary"
									value="NoResizing"
									bind:group={resize}
								/>
								<span class="ms-2">No resizing</span>
							</label>
						</div>
						<div class="form-control">
							<label class="label cursor-pointer justify-start">
								<input
									type="radio"
									name="resize"
									class="radio radio-primary"
									value="LongerSide"
									bind:group={resize}
								/>
								<span class="ms-2">Longer side</span>
							</label>
						</div>
						<div class="form-control">
							<label class="label cursor-pointer justify-start">
								<input
									type="radio"
									name="resize"
									class="radio radio-primary"
									value="ShorterSide"
									bind:group={resize}
								/>
								<span class="ms-2">Shorter side</span>
							</label>
						</div>
					</div>
					<label class="flex items-center gap-2">
						Resize to
						<input
							type="number"
							class="input input-bordered input-primary input-sm w-20"
							placeholder="1600"
							bind:value={resizeTo}
							disabled={resize === 'NoResizing'}
						/>
						<span>px</span>
					</label>
					<div class="form-control">
						<label class="label cursor-pointer justify-start">
							<input
								type="checkbox"
								class="checkbox checkbox-primary"
								bind:checked={isAllowEnlarging}
								disabled={resize === 'NoResizing'}
							/>
							<span class="ms-2">Allow enlarging</span>
						</label>
					</div>
				</div>
				<div class="border-b-2 border-gray-800 pb-3">
					<div class="py-2">Saves images to...</div>
					<div class="">
						<div class="form-control">
							<label class="label cursor-pointer justify-start">
								<input
									type="radio"
									name="saveTo"
									class="radio radio-primary"
									value="same-folder"
									bind:group={saveTo}
								/>
								<span class="ms-2">Same folder as image</span>
							</label>
						</div>
						<div class="form-control">
							<label class="label cursor-pointer justify-start">
								<input
									type="radio"
									name="saveTo"
									class="radio radio-primary"
									value="choose-folder"
									bind:group={saveTo}
								/>
								<span class="ms-2">Choose folder</span>
							</label>
						</div>
					</div>

					<div class="flex flex-row gap-1 items-center">
						<!-- svelte-ignore a11y_consider_explicit_label -->
						<button
							class="btn btn-primary btn-sm"
							disabled={saveTo === 'same-folder'}
							onclick={getFolder}
							><svg
								width="20px"
								height="20px"
								viewBox="0 0 24 24"
								xmlns="http://www.w3.org/2000/svg"
							>
								<path
									fill-rule="evenodd"
									clip-rule="evenodd"
									d="M1 5C1 3.34315 2.34315 2 4 2H8.55848C9.84977 2 10.9962 2.82629 11.4045 4.05132L11.7208 5H20C21.1046 5 22 5.89543 22 7V9.00961C23.1475 9.12163 23.9808 10.196 23.7695 11.3578L22.1332 20.3578C21.9603 21.3087 21.132 22 20.1654 22H3C1.89543 22 1 21.1046 1 20V5ZM20 9V7H11.7208C10.8599 7 10.0956 6.44914 9.82339 5.63246L9.50716 4.68377C9.37105 4.27543 8.98891 4 8.55848 4H4C3.44772 4 3 4.44772 3 5V12.2709L3.35429 10.588C3.54913 9.66249 4.36562 9 5.31139 9H20ZM3.36634 20C3.41777 19.9109 3.4562 19.8122 3.47855 19.706L5.31139 11L21 11H21.8018L20.1654 20L3.36634 20Z"
									fill="#000000"
								/>
							</svg></button
						>
						<input
							type="text"
							class="input input-bordered input-primary input-sm w-44"
							disabled={saveTo === 'same-folder'}
							bind:value={saveFolder}
						/>
					</div>
				</div>
			</div>
			<div class="px-2 py-2 w-auto">
				{#if inProgress}
					<button class="btn btn-error btn-sm" onclick={cancel} in:fade={{ duration: 50 }}
						>Cancel</button
					>
				{/if}
				{#if !inProgress}
					<button
						class="btn btn-primary btn-sm w-full"
						onclick={processImages}
						in:fade={{ duration: 50 }}>Start</button
					>
				{/if}
			</div>
		</section>
	{/if}
	{#if showAbout && !dropInProgress}
		<section class="flex-grow w-screen h-screen flex flex-col" in:fade={{ duration: 50 }}>
			<div
				class="flex-grow w-auto h-auto px-2 flex flex-col my-2 container mx-auto text-center justify-center"
			>
				<img src="/Icon_1024.png" class="w-40 mx-auto" alt="BlazingWebP logo" />
				<div class="text-6xl mb-2">BlazingWebP</div>
				<div class="mb-2">Version {version}</div>
				<div class="text-2xl mb-6">
					Your <span class="italic">blazingly fast</span>* WebP batch converter.
				</div>
				<!-- <div class="mb-4">Proudly made with</div>
				<div class="flex flex-row justify-center mb-12 gap-4">
					<a href="https://tauri.app" target="_blank"
						><img src="/tauri.svg" class="h-12" alt="Tauri logo" /></a
					>
					<a href="https://svelte.dev" target="_blank"
						><img src="/svelte.svg" class="h-12" alt="Svelte logo" /></a
					>
					<a href="https://vite.dev" target="_blank"
						><img src="/vite.svg" class="h-12" alt="Vite logo" /></a
					>
					<a href="https://www.rust-lang.org" target="_blank"
						><img src="/rust_ferris.svg" class="h-12" alt="Rust/Ferris logo" /></a
					>
					<a href="https://tailwindcss.com" target="_blank"
						><img src="/tailwind.svg" class="h-12" alt="Tailwind logo" /></a
					>
					<a href="https://daisyui.com" target="_blank"
						><img src="/daisyui.svg" class="h-12" alt="Daisy UI logo" /></a
					>
					<img src="/beer.svg" class="h-12" alt="Beer!" />
				</div> -->
				<div class="mb-2">
					This software is distributed "as is" with no guarantee according to the GPLv2 licence.
				</div>
				<div class="mb-2">
					Follow the team behind BlazingWebP on <a
						href="https://github.com/PRRPCHT/BlazingWebP"
						target="_blank"
						class="text-primary hover:text-secondary">GitHub</a
					>.
				</div>
			</div>
			<div class="flex justify-between py-2">
				<div class="w-48"></div>
				<button
					class="btn btn-primary btn-sm mx-2 w-24"
					onclick={toggleAbout}
					in:fade={{ duration: 50 }}>Close</button
				>
				<div class="w-48 text-end truncate mx-2 mt-3">*If you know, you know...</div>
			</div>
		</section>
	{/if}
	{#if dropInProgress}
		<section class="flex-grow w-screen h-screen flex flex-col" in:fade={{ duration: 50 }}>
			<div
				class="flex-grow w-auto h-auto px-2 flex flex-col my-2 container mx-auto text-center justify-center"
			>
				<div class="text-2xl mb-6">Drop your images here!</div>
				<div class="mb-2">Accepted formats: PNG, JPEG, WEBP.</div>
			</div>
		</section>
	{/if}
</main>

<style>
	.disabled-range {
		--range-shdw: dark-gray;
	}
</style>
