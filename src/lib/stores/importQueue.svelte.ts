import { listen } from "@tauri-apps/api/event";
import type { ImportQueueEvent } from "../types";
import { getBannerAutoClose } from "./bannerAutoClose.svelte";

type JobStatus = "queued" | "running" | "completed" | "failed";

export interface ImportJobState {
  jobId: string;
  status: JobStatus;
  total: number;
  current: number;
  currentTitle: string;
  succeeded: number;
  failed: number;
  errors: { title: string; message: string }[];
}

let jobs = $state<ImportJobState[]>([]);

function findJob(jobId: string): ImportJobState | undefined {
  return jobs.find((j) => j.jobId === jobId);
}

function handleEvent(event: ImportQueueEvent) {
  switch (event.type) {
    case "enqueued": {
      jobs.push({
        jobId: event.jobId,
        status: "queued",
        total: event.total,
        current: 0,
        currentTitle: "",
        succeeded: 0,
        failed: 0,
        errors: [],
      });
      break;
    }
    case "jobStarted": {
      const job = findJob(event.jobId);
      if (job) {
        job.status = "running";
        job.total = event.total;
      }
      break;
    }
    case "progress": {
      const job = findJob(event.jobId);
      if (job) {
        job.current = event.current;
        job.currentTitle = event.title;
      }
      break;
    }
    case "itemError": {
      const job = findJob(event.jobId);
      if (job) {
        job.errors.push({ title: event.title, message: event.message });
      }
      break;
    }
    case "jobCompleted": {
      const job = findJob(event.jobId);
      if (job) {
        job.status = "completed";
        job.succeeded = event.succeeded;
        job.failed = event.failed;
      }
      break;
    }
    case "jobFailed": {
      const job = findJob(event.jobId);
      if (job) {
        job.status = "failed";
      }
      break;
    }
  }
}

export function getJobs(): ImportJobState[] {
  return jobs;
}

export function getActiveJob(): ImportJobState | undefined {
  return jobs.find((j) => j.status === "running" || j.status === "queued");
}

export function dismissJob(jobId: string) {
  jobs = jobs.filter((j) => j.jobId !== jobId);
}

export function initImportQueueListener(onCompleted: () => void) {
  let autoCloseTimers: Record<string, ReturnType<typeof setTimeout>> = {};

  const unlistenPromise = listen<ImportQueueEvent>("import-queue", (event) => {
    handleEvent(event.payload);

    if (event.payload.type === "jobCompleted") {
      onCompleted();
      const { jobId, failed } = event.payload;
      const autoCloseSeconds = getBannerAutoClose();
      if (failed === 0 && autoCloseSeconds > 0) {
        autoCloseTimers[jobId] = setTimeout(() => {
          dismissJob(jobId);
          delete autoCloseTimers[jobId];
        }, autoCloseSeconds * 1000);
      }
    }
  });

  return () => {
    unlistenPromise.then((unlisten) => unlisten());
    Object.values(autoCloseTimers).forEach(clearTimeout);
    autoCloseTimers = {};
  };
}
