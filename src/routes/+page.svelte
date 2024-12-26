<script lang="ts">
	import { invoke } from '@tauri-apps/api/core';
	import { stat } from '@tauri-apps/plugin-fs';
	import type { Image } from '../types/types';
	import { open } from '@tauri-apps/plugin-dialog';
	import { basename, extname, dirname } from '@tauri-apps/api/path';

	let name = $state('');
	let greetMsg = $state('');
	let quality = $state(80);
	let resizing = $state('no-resizing');
	let resize = $state(1600);
	let compression = $state('lossless');
	let isAllowEnlarging = $state(false);
	let saveTo = $state('same-folder');
	let images = $state<Image[]>([]);
	// const img1: Image = {
	// 	filename: 'tartampion.jpg',
	// 	extension: 'jpg',
	// 	path: '/path/to/the folder/where the photo/is/',
	// 	originalSize: 678,
	// 	webpSize: 0,
	// 	processed: false
	// };
	// const img2: Image = {
	// 	filename: 'bidule.jpg',
	// 	extension: 'jpg',
	// 	path: '/path/to/the folder/where the photo/is/',
	// 	originalSize: 567,
	// 	webpSize: 188,
	// 	processed: true
	// };
	// images.push(img2);
	// images.push(img2);
	// images.push(img2);
	// images.push(img1);
	// images.push(img1);
	// images.push(img1);
	// images.push(img1);
	// images.push(img1);
	// images.push(img1);
	// images.push(img1);
	// images.push(img1);
	// images.push(img1);

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
			// Convert size to kilobytes
			fileSizeKB = Math.round(fileMetadata.size / 1024);
		} else {
			fileSizeKB = -1;
		}
		return fileSizeKB;
	}

	async function greet(event: Event) {
		event.preventDefault();
		// Learn more about Tauri commands at https://tauri.app/develop/calling-rust/
		greetMsg = await invoke('greet', { name });
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
					filename: filename,
					extension: extension,
					path: filepath,
					originalSize: await getFileSize(file),
					webpSize: 0,
					processed: false
				};
				images.push(newImage);
			});
		}
		console.log(images);
	}

	function clear() {
		images = [];
	}
</script>

<main class="flex">
	<section class="flex-grow min-w-96 h-screen flex flex-col">
		<div class="flex-grow w-auto h-auto px-2 flex flex-col overflow-y-auto my-2">
			{#each images as image}
				<div class="flex justify-between border-b-[1px] border-gray-800 pb-2">
					<div>
						<div>{image.filename + image.extension}</div>
						<div>{image.path}</div>
					</div>
					<div>
						<div
							class="text-right"
							class:text-green-500={image.processed}
							class:text-red-500={!image.processed}
						>
							{image.processed ? 'DONE' : 'TODO'}
						</div>
						<div>
							{image.originalSize} KB {image.processed ? '> ' + image.webpSize + ' KB' : ''}
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
			<div class="border-b-[1px] border-gray-800 pb-3">
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
			<div class="border-b-[1px] border-gray-800 pb-3">
				<div class="py-2">Image resize</div>
				<div class="">
					<div class="form-control">
						<label class="label cursor-pointer justify-start">
							<input
								type="radio"
								name="resize"
								class="radio radio-primary"
								value="no-resizing"
								bind:group={resizing}
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
								value="longer-side"
								bind:group={resizing}
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
								value="shorter-side"
								bind:group={resizing}
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
						bind:value={resize}
						disabled={resizing === 'no-resizing'}
					/>
					<span>px</span>
				</label>
				<div class="form-control">
					<label class="label cursor-pointer justify-start">
						<input
							type="checkbox"
							class="checkbox checkbox-primary"
							bind:checked={isAllowEnlarging}
							disabled={resizing === 'no-resizing'}
						/>
						<span class="ms-2">Allow enlarging</span>
					</label>
				</div>
			</div>
			<div class="border-b-[1px] border-gray-800 pb-3">
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
					/>
				</div>
			</div>
		</div>
		<div class="px-2 py-2 w-auto"><button class="btn btn-primary btn-sm w-full">Start</button></div>
	</section>
</main>

<style>
	.disabled-range {
		--range-shdw: dark-gray;
	}
</style>
