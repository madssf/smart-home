import type {useTransition} from "@remix-run/react";

export const useSubmissionStatus = (transition: ReturnType<typeof useTransition>, document: {id: string} | undefined) => {
    const isCreating = transition.submission?.formData.get("intent") === "create" &&
        (transition.submission?.formData.get('id') ?? undefined) === document?.id;
    const isUpdating = transition.submission?.formData.get("intent") === "update" &&
        (transition.submission?.formData.get('id') ?? undefined) === document?.id;
    const isDeleting = transition.submission?.formData.get("intent") === "delete" &&
        (transition.submission?.formData.get('id') ?? undefined) === document?.id;
    const isNew = !document;
    return {isCreating, isDeleting, isUpdating, isNew};
};
