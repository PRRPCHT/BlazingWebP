export type Image = {
    fullPath: string,
    filename: string,
    extension: string,
    path: string,
    originalSize: number,
    webpSize: number,
    processed: boolean,
}

export type Parameters = {
    isLossless: boolean,
    quality: number,
    resize: string,
    resizeTo: number,
    isEnlargingAllowed: boolean,
    saveFolder: string,
}