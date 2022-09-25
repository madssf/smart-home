export const collections = {
    schedules: (userId: string) => generateCollectionPath(userId, 'schedules')
}

const generateCollectionPath = (userId: string, collection: string) => {
    return `users/${userId}/${collection}`
}