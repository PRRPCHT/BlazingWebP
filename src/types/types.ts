export enum Status {
    TODO = "TODO",
    SUCCESS = "SUCCESS",
    ERROR = "ERROR",
}

export type Image = {
    fullPath: string,
    filename: string,
    extension: string,
    path: string,
    originalSize: number,
    webpSize: number,
    status: Status,
    errorMessage: string,
}

export type Parameters = {
    isLossless: boolean,
    quality: number,
    resize: string,
    resizeTo: number,
    isEnlargingAllowed: boolean,
    saveFolder: string,
}

export type Success = {
    fullPath: string,
    size: number,
}

export type ProcessError = {
    fullPath: string,
    error: string,
}