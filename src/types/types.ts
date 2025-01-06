export enum Status {
    TODO = "TO DO",
    SUCCESS = "DONE",
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
    inProgress: boolean
}

export enum ImageErrorType {
    WRONG_FORMAT,
    ALREADY_EXISTS,
}

export type ImageError = {
    type: ImageErrorType,
    file: string,
}

export type Parameters = {
    action: string,
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