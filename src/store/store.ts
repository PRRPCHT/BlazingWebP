import { writable } from 'svelte/store';
import type { Image } from '../types/types';

export const images = writable<Image[]>([]);