import { HttpErrorResponse } from '@angular/common/http';
import { inject } from '@angular/core';
import { ActivatedRouteSnapshot, Router } from '@angular/router';

import { EMPTY, Observable, catchError, of } from 'rxjs';

import { ICvResumeVersion } from '../cv-resume-version.model';
import { CvResumeVersionService } from '../service/cv-resume-version.service';

const cvResumeVersionResolve = (route: ActivatedRouteSnapshot): Observable<null | ICvResumeVersion> => {
  const { id } = route.params;
  if (id) {
    const router = inject(Router);
    const service = inject(CvResumeVersionService);
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

export default cvResumeVersionResolve;
