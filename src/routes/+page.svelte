<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { stat } from '@tauri-apps/plugin-fs';
	import { type Image, type Parameters, type Success, Status } from '../types/types';
	import { open } from '@tauri-apps/plugin-dialog';
	import { basename, extname, dirname } from '@tauri-apps/api/path';
	import { listen } from '@tauri-apps/api/event';

	let quality = $state(80);
	let resize = $state('NoResizing');
	let resizeTo = $state(1600);
	let compression = $state('lossless');
	let isAllowEnlarging = $state(false);
	let saveTo = $state('same-folder');
	let saveFolder = $state('');
	let images = $state<Image[]>([]);

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
		let parameters: Parameters = {
			isLossless: compression === 'lossless',
			quality: quality,
			resize: resize,
			resizeTo: resizeTo,
			isEnlargingAllowed: isAllowEnlarging,
			saveFolder: saveFolder
		};
		await invoke('process', { images, parameters });
	}

	async function addFiles() {
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
			files.forEach(async (file) => {
				let { filepath, filename, extension } = await extractFileDetails(file);
				let newImage: Image = {
					fullPath: file,
					filename: filename,
					extension: extension,
					path: filepath,
					originalSize: await getFileSize(file),
					webpSize: 0,
					status: Status.TODO
				};
				images.push(newImage);
			});
		}
		console.log(images);
	}

	function clear() {
		images = [];
	}

	function updateList(success: Success) {
		console.log('Updating for ' + success.fullPath);
		images.forEach((image) => {
			console.log('Path: ' + image.fullPath);
			if (image.fullPath == success.fullPath) {
				image.status = Status.SUCCESS;
				image.webpSize = success.size;
				console.log('Updated in list');
			}
		});
	}

	listen<Success>('success', (event) => {
		console.log(`Succes for ${event.payload.fullPath} with size ${event.payload.size}`);
		updateList(event.payload);
	});
</script>

<main class="flex">
	<section class="flex-grow min-w-96 h-screen flex flex-col">
		<div class="flex-grow w-auto h-auto px-2 flex flex-col overflow-y-auto my-2">
			{#each images as image}
				<div class="flex justify-between border-b-2 border-gray-800 py-3 first:pt-0">
					<div>
						<div class="py-1 first:pt-0">{image.filename + image.extension}</div>
						<div>{image.path}</div>
					</div>
					<div>
						<div
							class="text-right py-1 first:pt-0"
							class:text-green-500={image.status == Status.SUCCESS}
							class:text-amber-500={image.status == Status.TODO}
							class:text-red-500={image.status == Status.ERROR}
						>
							{image.status}
						</div>
						<div>
							{image.originalSize} KB {image.status == Status.SUCCESS
								? '> ' + image.webpSize + ' KB'
								: ''}
						</div>
					</div>
				</div>
			{/each}
		</div>

		<div class="flex justify-between py-2">
			<button class="btn btn-primary btn-sm mx-2" onclick={addFiles}>Add images</button>
			<button class="btn btn-neutral btn-sm" onclick={clear}>Clear</button>
		</div>
	</section>
	<section class="min-w-72 w-72 h-screen flex flex-col">
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
						disabled={resize === 'no-resizing'}
					/>
					<span>px</span>
				</label>
				<div class="form-control">
					<label class="label cursor-pointer justify-start">
						<input
							type="checkbox"
							class="checkbox checkbox-primary"
							bind:checked={isAllowEnlarging}
							disabled={resize === 'no-resizing'}
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
							<span class="ms-2">Same folder</span>
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

				<div class="flex flex-row gap-2 items-center">
					<button class="btn btn-primary btn-sm" disabled={saveTo === 'same-folder'}>Choose</button>
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
			<button class="btn btn-primary btn-sm w-full" onclick={processImages}>Start</button>
		</div>
	</section>
</main>

<style>
	.disabled-range {
		--range-shdw: dark-gray;
	}
</style>
