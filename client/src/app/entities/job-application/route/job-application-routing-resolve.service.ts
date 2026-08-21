import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, Router } from '@angular/router';

import { EMPTY, Observable, catchError, of } from 'rxjs';

import { IJobApplication } from '../job-application.model';
import { JobApplicationService } from '../service/job-application.service';

const jobApplicationResolve = (route: ActivatedRouteSnapshot): Observable<null | IJobApplication> => {
  const { id } = route.params;
  if (id) {
    const router = inject(Router);
    const service = inject(JobApplicationService);
    return service.find(id).pipe(
      catchError((error: HttpErrorResponse) => {
        if (error.status === 404) {
          router.navigate(['404']);
        } else {
          router.navigate(['error']);
        }
        return EMPTY;
      }),
    );
  }

  return of(null);
};

export default jobApplicationResolve;
