import { HttpClient, HttpResponse, httpResource } from '@angular/common/http';
import { Injectable, computed, inject, signal } from '@angular/core';

import dayjs from 'dayjs/esm';
import { Observable, map } from 'rxjs';

import { ApplicationConfigService } from 'app/core/config/application-config.service';
import { createRequestOption } from 'app/core/request/request-util';
import { isPresent } from 'app/core/util/operators';
import { IJobApplication, NewJobApplication } from '../job-application.model';

export type PartialUpdateJobApplication = Partial<IJobApplication> & Pick<IJobApplication, 'id'>;

type RestOf<T extends IJobApplication | NewJobApplication> = Omit<T, 'createdAt' | 'updatedAt' | 'appliedAt'> & {
  createdAt?: string | null;
  updatedAt?: string | null;
  appliedAt?: string | null;
};

export type RestJobApplication = RestOf<IJobApplication>;

export type NewRestJobApplication = RestOf<NewJobApplication>;

export type PartialUpdateRestJobApplication = RestOf<PartialUpdateJobApplication>;

@Injectable()
export class JobApplicationsService {
  readonly jobApplicationsParams = signal<Record<string, string | number | boolean | readonly (string | number | boolean)[]> | undefined>(
    undefined,
  );
  readonly jobApplicationsResource = httpResource<RestJobApplication[]>(() => {
    const params = this.jobApplicationsParams();
    if (!params) {
      return undefined;
    }
    return { url: this.resourceUrl, params };
  });
  /**
   * This signal holds the list of jobApplication that have been fetched. It is updated when the jobApplicationsResource emits a new value.
   * In case of error while fetching the jobApplications, the signal is set to an empty array.
   */
  readonly jobApplications = computed(() =>
    (this.jobApplicationsResource.hasValue() ? this.jobApplicationsResource.value() : []).map(item => this.convertValueFromServer(item)),
  );
  protected readonly applicationConfigService = inject(ApplicationConfigService);
  protected readonly resourceUrl = this.applicationConfigService.getEndpointFor('api/job-applications');

  protected convertValueFromServer(restJobApplication: RestJobApplication): IJobApplication {
    return {
      ...restJobApplication,
      createdAt: restJobApplication.createdAt ? dayjs(restJobApplication.createdAt) : undefined,
      updatedAt: restJobApplication.updatedAt ? dayjs(restJobApplication.updatedAt) : undefined,
      appliedAt: restJobApplication.appliedAt ? dayjs(restJobApplication.appliedAt) : undefined,
    };
  }
}

@Injectable({ providedIn: 'root' })
export class JobApplicationService extends JobApplicationsService {
  protected readonly http = inject(HttpClient);

  create(jobApplication: NewJobApplication): Observable<IJobApplication> {
    const copy = this.convertValueFromClient(jobApplication);
    return this.http.post<RestJobApplication>(this.resourceUrl, copy).pipe(map(res => this.convertResponseFromServer(res)));
  }

  update(jobApplication: IJobApplication): Observable<IJobApplication> {
    const copy = this.convertValueFromClient(jobApplication);
    return this.http
      .put<RestJobApplication>(`${this.resourceUrl}/${encodeURIComponent(this.getJobApplicationIdentifier(jobApplication))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  partialUpdate(jobApplication: PartialUpdateJobApplication): Observable<IJobApplication> {
    const copy = this.convertValueFromClient(jobApplication);
    return this.http
      .patch<RestJobApplication>(`${this.resourceUrl}/${encodeURIComponent(this.getJobApplicationIdentifier(jobApplication))}`, copy)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  find(id: number): Observable<IJobApplication> {
    return this.http
      .get<RestJobApplication>(`${this.resourceUrl}/${encodeURIComponent(id)}`)
      .pipe(map(res => this.convertResponseFromServer(res)));
  }

  query(req?: any): Observable<HttpResponse<IJobApplication[]>> {
    const options = createRequestOption(req);
    return this.http
      .get<RestJobApplication[]>(this.resourceUrl, { params: options, observe: 'response' })
      .pipe(map(res => res.clone({ body: this.convertResponseArrayFromServer(res.body!) })));
  }

  delete(id: number): Observable<undefined> {
    return this.http.delete<undefined>(`${this.resourceUrl}/${encodeURIComponent(id)}`);
  }

  getJobApplicationIdentifier(jobApplication: Pick<IJobApplication, 'id'>): number {
    return jobApplication.id;
  }

  compareJobApplication(o1: Pick<IJobApplication, 'id'> | null, o2: Pick<IJobApplication, 'id'> | null): boolean {
    return o1 && o2 ? this.getJobApplicationIdentifier(o1) === this.getJobApplicationIdentifier(o2) : o1 === o2;
  }

  addJobApplicationToCollectionIfMissing<Type extends Pick<IJobApplication, 'id'>>(
    jobApplicationCollection: Type[],
    ...jobApplicationsToCheck: (Type | null | undefined)[]
  ): Type[] {
    const jobApplications: Type[] = jobApplicationsToCheck.filter(isPresent);
    if (jobApplications.length > 0) {
      const jobApplicationCollectionIdentifiers = jobApplicationCollection.map(jobApplicationItem =>
        this.getJobApplicationIdentifier(jobApplicationItem),
      );
      const jobApplicationsToAdd = jobApplications.filter(jobApplicationItem => {
        const jobApplicationIdentifier = this.getJobApplicationIdentifier(jobApplicationItem);
        if (jobApplicationCollectionIdentifiers.includes(jobApplicationIdentifier)) {
          return false;
        }
        jobApplicationCollectionIdentifiers.push(jobApplicationIdentifier);
        return true;
      });
      return [...jobApplicationsToAdd, ...jobApplicationCollection];
    }
    return jobApplicationCollection;
  }

  protected convertValueFromClient<T extends IJobApplication | NewJobApplication | PartialUpdateJobApplication>(
    jobApplication: T,
  ): RestOf<T> {
    return {
      ...jobApplication,
      createdAt: jobApplication.createdAt?.toJSON() ?? null,
      updatedAt: jobApplication.updatedAt?.toJSON() ?? null,
      appliedAt: jobApplication.appliedAt?.toJSON() ?? null,
    };
  }

  protected convertResponseFromServer(res: RestJobApplication): IJobApplication {
    return this.convertValueFromServer(res);
  }

  protected convertResponseArrayFromServer(res: RestJobApplication[]): IJobApplication[] {
    return res.map(item => this.convertValueFromServer(item));
  }
}
